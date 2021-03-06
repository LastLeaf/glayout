use std::collections::HashMap;
use std::ptr;
use std::mem;
use std::ffi::{CStr, CString};
use std::os::raw::{c_void, c_char};
use super::gl;
use super::gl::Gles2 as Gl;

const GL_DRAW_RECT_MAX: i32 = super::GL_DRAW_RECT_MAX;
const TEXTURE_MAX: i32 = super::TEXTURE_MAX;

macro_rules! paint {
    ($canvas_index: expr, $f: expr) => {
        let w = super::MAIN_LOOP_WINDOWS.read().unwrap();
        let w = w.get(&$canvas_index).unwrap();
        let mut w = w.lock().unwrap();
        w.painting_thread.append_command(super::PaintingCommand::CustomCommand(Box::new($f)));
    }
}

fn string_from_buffer(buf: &[u8]) -> String {
    unsafe { CStr::from_ptr(buf.as_ptr() as *const i8).to_string_lossy().into_owned() }
}

fn buffer_from_str(str: &str) -> *mut c_char {
    CString::new(str).unwrap().into_raw()
}

fn create_shader(ctx: &mut Gl, shader_type: u32, src: &str) -> u32 {
    let src = CString::new(src).unwrap();
    let ptr = Box::into_raw(Box::new(src.as_ptr()));
    unsafe {
        let shader = ctx.CreateShader(shader_type);
        ctx.ShaderSource(shader, 1, ptr, ptr::null());
        ctx.CompileShader(shader);
        Box::from_raw(ptr);
        let mut ret = 0;
        ctx.GetShaderiv(shader, gl::COMPILE_STATUS, &mut ret);
        if ret == gl::FALSE as i32 {
            let mut buf: [u8; 4096] = [0; 4096];
            let mut buf_len: i32 = 0;
            ctx.GetShaderInfoLog(shader, 4096, &mut buf_len, buf.as_mut_ptr() as *mut i8);
            error!("Compiling shader failed: {}", string_from_buffer(&buf));
            ctx.DeleteShader(shader);
            panic!();
        }
        shader
    }
}
fn create_program(ctx: &mut Gl, vs: &str, fs: &str) -> u32 {
    let vs = create_shader(ctx, gl::VERTEX_SHADER, vs);
    let fs = create_shader(ctx, gl::FRAGMENT_SHADER, fs);
    let program = unsafe { ctx.CreateProgram() };
    unsafe {
        ctx.AttachShader(program, vs);
        ctx.AttachShader(program, fs);
        ctx.LinkProgram(program);
        let mut ret = 0;
        ctx.GetProgramiv(program, gl::LINK_STATUS, &mut ret);
        if ret == gl::FALSE as i32 {
            let mut buf: [u8; 4096] = [0; 4096];
            let mut buf_len: i32 = 0;
            ctx.GetShaderInfoLog(program, 4096, &mut buf_len, buf.as_mut_ptr() as *mut i8);
            error!("Linking shader program failed: {}", string_from_buffer(&buf));
            ctx.DeleteProgram(program);
            panic!();
        }
    }
    program
}

fn generate_index_buf_content(buf: &mut [u16; GL_DRAW_RECT_MAX as usize * 6]) {
    for i in 0..GL_DRAW_RECT_MAX {
        let base4 = (i * 4) as u16;
        let base6 = (i * 6) as usize;
        buf[base6] = base4;
        buf[base6 + 1] = base4 + 1;
        buf[base6 + 2] = base4 + 2;
        buf[base6 + 3] = base4;
        buf[base6 + 4] = base4 + 2;
        buf[base6 + 5] = base4 + 3;
    }
}

pub struct TexManager {
    width: i32,
    height: i32,
    pixel_ratio: f64,
    u_area_size: i32,
    u_color: i32,
    u_alpha: i32,
    tex_size: i32,
    tex_count: i32,
    img_shader_program: u32,
    tex_pos_gl_buf: u32,
    tex_pos_buf: Box<[f32; GL_DRAW_RECT_MAX as usize * 8]>,
    draw_pos_gl_buf: u32,
    draw_pos_buf: Box<[f32; GL_DRAW_RECT_MAX as usize * 8]>,
    tex_index_gl_buf: u32,
    tex_index_buf: Box<[f32; GL_DRAW_RECT_MAX as usize * 4]>,
    temp_framebuffer: u32,
    temp_tex: u32,
    tex_map: HashMap<i32, u32>,
    binded_rendering_target_stack: Vec<(i32, i32, i32)>,
}
impl TexManager {
    pub fn new(ctx: &mut Gl, tex_size: i32, tex_count: i32) -> Self {
        unsafe {
            let img_shader_program = create_program(ctx, include_str!("../glsl/img.v.glsl"), include_str!("../glsl/img.f.glsl"));
            ctx.UseProgram(img_shader_program);

            // create gl buffers
            let mut gl_bufs = [0; 4];
            ctx.GenBuffers(4, gl_bufs.as_mut_ptr());

            // the texture position buffer
            let tex_pos_gl_buf = gl_bufs[0];
            let tex_pos_buf = Box::new([0. as f32; GL_DRAW_RECT_MAX as usize * 8]);
            ctx.BindBuffer(gl::ARRAY_BUFFER, tex_pos_gl_buf);
            ctx.BufferData(gl::ARRAY_BUFFER, GL_DRAW_RECT_MAX as isize * 8 * mem::size_of::<f32>() as isize, tex_pos_buf.as_ptr() as *const c_void, gl::STREAM_DRAW);
            let a_tex_pos = ctx.GetAttribLocation(img_shader_program, buffer_from_str("aTexPos")) as u32;
            ctx.EnableVertexAttribArray(a_tex_pos);
            ctx.VertexAttribPointer(a_tex_pos, 2, gl::FLOAT, gl::FALSE, 0, 0 as *const c_void);

            // the draw position buffer
            let draw_pos_gl_buf = gl_bufs[1];
            let draw_pos_buf = Box::new([0. as f32; GL_DRAW_RECT_MAX as usize * 8]);
            ctx.BindBuffer(gl::ARRAY_BUFFER, draw_pos_gl_buf);
            ctx.BufferData(gl::ARRAY_BUFFER, GL_DRAW_RECT_MAX as isize * 8 * mem::size_of::<f32>() as isize, draw_pos_buf.as_ptr() as *const c_void, gl::STREAM_DRAW);
            let a_draw_pos = ctx.GetAttribLocation(img_shader_program, buffer_from_str("aDrawPos")) as u32;
            ctx.EnableVertexAttribArray(a_draw_pos);
            ctx.VertexAttribPointer(a_draw_pos, 2, gl::FLOAT, gl::FALSE, 0, 0 as *const c_void);

            // the draw position buffer
            let tex_index_gl_buf = gl_bufs[2];
            let tex_index_buf = Box::new([0. as f32; GL_DRAW_RECT_MAX as usize * 4]);
            ctx.BindBuffer(gl::ARRAY_BUFFER, tex_index_gl_buf);
            ctx.BufferData(gl::ARRAY_BUFFER, GL_DRAW_RECT_MAX as isize * 4 * mem::size_of::<f32>() as isize, tex_index_buf.as_ptr() as *const c_void, gl::STREAM_DRAW);
            let a_tex_index = ctx.GetAttribLocation(img_shader_program, buffer_from_str("aTexIndex")) as u32;
            ctx.EnableVertexAttribArray(a_tex_index);
            ctx.VertexAttribPointer(a_tex_index, 1, gl::FLOAT, gl::FALSE, 0, 0 as *const c_void);

            // the element indices buffer
            let index_gl_buf = gl_bufs[3];
            let mut index_buf = Box::new([0 as u16; GL_DRAW_RECT_MAX as usize * 6]);
            generate_index_buf_content(&mut index_buf);
            ctx.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_gl_buf);
            ctx.BufferData(gl::ELEMENT_ARRAY_BUFFER, GL_DRAW_RECT_MAX as isize * 6 * mem::size_of::<u16>() as isize, index_buf.as_ptr() as *const c_void, gl::STATIC_DRAW);

            // the temp framebuffer and texture
            let mut gl_framebuffers = [0 as u32; 1];
            ctx.GenFramebuffers(1, gl_framebuffers.as_mut_ptr());
            let temp_framebuffer = gl_framebuffers[0];
            let mut gl_textures = [0 as u32; 1];
            ctx.GenTextures(1, gl_textures.as_mut_ptr());
            let temp_tex = gl_textures[0];
            ctx.BindTexture(gl::TEXTURE_2D, temp_tex);
            ctx.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            ctx.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            ctx.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            ctx.TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, 256, 256, 0, gl::RGBA, gl::UNSIGNED_BYTE, ptr::null());

            // get other vars
            let u_area_size = ctx.GetUniformLocation(img_shader_program, buffer_from_str("uAreaSize"));
            let u_color = ctx.GetUniformLocation(img_shader_program, buffer_from_str("uColor"));
            let u_alpha = ctx.GetUniformLocation(img_shader_program, buffer_from_str("uAlpha"));
            ctx.Uniform4f(u_color, 0., 0., 0., 1.);
            ctx.Uniform1f(u_alpha, 1.);

            // bind default tex
            for i in 0..TEXTURE_MAX {
                ctx.ActiveTexture(gl::TEXTURE0 + i as u32);
                ctx.BindTexture(gl::TEXTURE_2D, temp_tex);
                let u_tex_i = ctx.GetUniformLocation(img_shader_program, buffer_from_str(&(String::from("uTex") + &i.to_string())));
                ctx.Uniform1i(u_tex_i, i);
            }

            Self {
                width: 1,
                height: 1,
                pixel_ratio: 1.0,
                u_area_size,
                u_color,
                u_alpha,
                tex_size,
                tex_count,
                img_shader_program,
                tex_pos_gl_buf,
                tex_pos_buf,
                draw_pos_gl_buf,
                draw_pos_buf,
                tex_index_gl_buf,
                tex_index_buf,
                temp_framebuffer,
                temp_tex,
                tex_map: HashMap::new(),
                binded_rendering_target_stack: vec![],
            }
        }
    }
}

impl TexManager {
    pub fn set_tex_draw_size(&mut self, ctx: &mut Gl, w: i32, h: i32, pixel_ratio: f64) {
        self.width = w;
        self.height = h;
        self.pixel_ratio = pixel_ratio;
        unsafe {
            ctx.Viewport(0, 0, (w as f64 * pixel_ratio).round() as i32, (h as f64 * pixel_ratio).round() as i32);
            ctx.Uniform3f(self.u_area_size, w as f32, h as f32, 1.);
        }
    }

    fn tex_create(&mut self, ctx: &mut Gl, w: i32, h: i32, buf: &Vec<u8>, tex_id: i32) {
        let tex = if tex_id < 0 {
            self.temp_tex
        } else {
            let mut gl_textures = [0 as u32; 1];
            unsafe { ctx.GenTextures(1, gl_textures.as_mut_ptr()) };
            let tex = gl_textures[0];
            self.tex_map.insert(tex_id, tex);
            tex
        };
        unsafe {
            ctx.BindTexture(gl::TEXTURE_2D, tex);
            ctx.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            ctx.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            ctx.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            ctx.TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, w, h, 0, gl::RGBA, gl::UNSIGNED_BYTE, buf.as_ptr() as *const c_void);
        }
    }
}

pub fn tex_create(canvas_index: i32, width: i32, height: i32, buf: Vec<u8>, tex_id: i32) {
    paint!(canvas_index, move |ctx, tex_manager| {
        // println!("tex_create {:?}", (canvas_index, tex_id, width, height));
        tex_manager.tex_create(ctx, width, height, &buf, tex_id);
    });
}
pub fn tex_rewrite(canvas_index: i32, buf: Vec<u8>, tex_id: i32, left: i32, top: i32, width: i32, height: i32) {
    paint!(canvas_index, move |ctx, tex_manager| {
        // println!("tex_rewrite {:?}", (canvas_index, tex_id, left, top, width, height));
        let ptr = buf.as_ptr() as *const c_void;
        unsafe {
            ctx.BindTexture(gl::TEXTURE_2D, if tex_id < 0 { tex_manager.temp_tex } else { tex_manager.tex_map[&tex_id] });
            ctx.TexSubImage2D(gl::TEXTURE_2D, 0, left, top, width, height, gl::RGBA, gl::UNSIGNED_BYTE, ptr);
            ctx.BindTexture(gl::TEXTURE_2D, 0);
        }
    });
}
pub fn tex_copy(canvas_index: i32, dest_tex_id: i32, dest_left: i32, dest_top: i32, src_left: i32, src_top: i32, width: i32, height: i32) {
    paint!(canvas_index, move |ctx, tex_manager| {
        unsafe {
            ctx.BindTexture(gl::TEXTURE_2D, tex_manager.tex_map[&dest_tex_id]);
            ctx.CopyTexSubImage2D(gl::TEXTURE_2D, 0, dest_left, dest_top, src_left, src_top, width, height);
            ctx.BindTexture(gl::TEXTURE_2D, 0);
        }
    });
}

fn tex_set_rendering_target(ctx: &mut Gl, tex_manager: &mut TexManager, tex_id: i32, width: i32, height: i32, need_clear: bool) {
    if tex_id < -1 {
        unsafe {
            ctx.BindFramebuffer(gl::FRAMEBUFFER, 0);
            ctx.UseProgram(tex_manager.img_shader_program);
            ctx.Viewport(0, 0, (tex_manager.width as f64 * tex_manager.pixel_ratio) as i32, (tex_manager.height as f64 * tex_manager.pixel_ratio) as i32);
            ctx.Uniform3f(tex_manager.u_area_size, tex_manager.width as f32, tex_manager.height as f32, 1.);
        }
    } else {
        unsafe {
            let tex = if tex_id < 0 { tex_manager.temp_tex } else { tex_manager.tex_map[&tex_id] };
            ctx.BindFramebuffer(gl::FRAMEBUFFER, tex_manager.temp_framebuffer);
            ctx.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, tex, 0);
            ctx.UseProgram(tex_manager.img_shader_program);
            ctx.Viewport(0, 0, width, height);
            ctx.Uniform3f(tex_manager.u_area_size, width as f32, height as f32, -1.);
            ctx.ClearColor(0., 0., 0., 0.);
            if need_clear { ctx.Clear(gl::COLOR_BUFFER_BIT); }
        }
    }
}
fn tex_bind_rendering_target_self(ctx: &mut Gl, tex_manager: &mut TexManager, tex_id: i32, width: i32, height: i32) {
    tex_manager.binded_rendering_target_stack.push((tex_id, width, height));
    let tex = if tex_id < 0 { tex_manager.temp_tex } else { tex_manager.tex_map[&tex_id] };
    unsafe {
        ctx.BindTexture(gl::TEXTURE_2D, tex);
        ctx.TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width, height, 0, gl::RGBA, gl::UNSIGNED_BYTE, ptr::null());
        ctx.BindTexture(gl::TEXTURE_2D, 0);
    }
    tex_set_rendering_target(ctx, tex_manager, tex_id, width, height, true);
}
pub fn tex_bind_rendering_target(canvas_index: i32, tex_id: i32, width: i32, height: i32) {
    paint!(canvas_index, move |ctx, tex_manager| {
        // println!("tex_bind_rendering_target {:?}", (canvas_index, tex_id, width, height));
        tex_bind_rendering_target_self(ctx, tex_manager, tex_id, width, height)
    });
}
fn tex_unbind_rendering_target_self(ctx: &mut Gl, tex_manager: &mut TexManager) {
    tex_manager.binded_rendering_target_stack.pop();
    let x = match tex_manager.binded_rendering_target_stack.last().clone() {
        None => (-2, 0, 0),
        Some(x) => *x
    };
    let (tex_id, width, height) = x;
    tex_set_rendering_target(ctx, tex_manager, tex_id, width, height, false);
}
pub fn tex_unbind_rendering_target(canvas_index: i32) {
    paint!(canvas_index, move |ctx, tex_manager| {
        // println!("tex_unbind_rendering_target {:?}", (canvas_index));
        tex_unbind_rendering_target_self(ctx, tex_manager);
    });
}

pub fn tex_create_empty(canvas_index: i32, tex_id: i32, width: i32, height: i32) {
    paint!(canvas_index, move |ctx, tex_manager| {
        // println!("tex_create_empty {:?}", (canvas_index, tex_id, width, height));
        let mut gl_textures = [0 as u32; 1];
        unsafe { ctx.GenTextures(1, gl_textures.as_mut_ptr()) };
        let tex = gl_textures[0];
        tex_manager.tex_map.insert(tex_id, tex);
        unsafe {
            ctx.BindTexture(gl::TEXTURE_2D, tex);
            ctx.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            ctx.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            ctx.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            if width > 0 && height > 0 {
                ctx.TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width, height, 0, gl::RGBA, gl::UNSIGNED_BYTE, ptr::null());
            }
            tex_bind_rendering_target_self(ctx, tex_manager, tex_id, width, height);
            tex_unbind_rendering_target_self(ctx, tex_manager);
            ctx.BindTexture(gl::TEXTURE_2D, 0);
        }
    });
}
pub fn tex_delete(canvas_index: i32, tex_id: i32) {
    paint!(canvas_index, move |ctx, tex_manager| {
        // println!("tex_delete {:?}", (canvas_index, tex_id));
        unsafe {
            let texture = tex_manager.tex_map.remove(&tex_id).unwrap();
            ctx.DeleteTextures(1, &texture);
        }
    });
}
pub fn tex_draw(canvas_index: i32, draw_index: i32, tex_shader_index: i32, normalized_tex_x: f32, normalized_tex_y: f32, normalized_tex_w: f32, normalized_tex_h: f32, x: f32, y: f32, w: f32, h: f32) {
    paint!(canvas_index, move |_ctx, tex_manager| {
        // println!("tex_draw {:?}", (canvas_index, draw_index, tex_shader_index, normalized_tex_x, normalized_tex_y, normalized_tex_w, normalized_tex_h, x, y, w, h));
        let tex_pos_buf = &mut *tex_manager.tex_pos_buf;
        let draw_pos_buf = &mut *tex_manager.draw_pos_buf;
        let tex_index_buf = &mut *tex_manager.tex_index_buf;
        let draw_index_8 = draw_index as usize * 8;
        let draw_index_4 = draw_index as usize * 4;
        tex_pos_buf[draw_index_8 + 0] = normalized_tex_x;
        tex_pos_buf[draw_index_8 + 1] = normalized_tex_y;
        tex_pos_buf[draw_index_8 + 2] = normalized_tex_x;
        tex_pos_buf[draw_index_8 + 3] = normalized_tex_y + normalized_tex_h;
        tex_pos_buf[draw_index_8 + 4] = normalized_tex_x + normalized_tex_w;
        tex_pos_buf[draw_index_8 + 5] = normalized_tex_y + normalized_tex_h;
        tex_pos_buf[draw_index_8 + 6] = normalized_tex_x + normalized_tex_w;
        tex_pos_buf[draw_index_8 + 7] = normalized_tex_y;
        draw_pos_buf[draw_index_8 + 0] = x;
        draw_pos_buf[draw_index_8 + 1] = y;
        draw_pos_buf[draw_index_8 + 2] = x;
        draw_pos_buf[draw_index_8 + 3] = y + h;
        draw_pos_buf[draw_index_8 + 4] = x + w;
        draw_pos_buf[draw_index_8 + 5] = y + h;
        draw_pos_buf[draw_index_8 + 6] = x + w;
        draw_pos_buf[draw_index_8 + 7] = y;
        tex_index_buf[draw_index_4 + 0] = tex_shader_index as f32;
        tex_index_buf[draw_index_4 + 1] = tex_shader_index as f32;
        tex_index_buf[draw_index_4 + 2] = tex_shader_index as f32;
        tex_index_buf[draw_index_4 + 3] = tex_shader_index as f32;
    });
}
pub fn tex_set_active_texture(canvas_index: i32, tex_shader_index: i32, tex_id: i32) {
    paint!(canvas_index, move |ctx, tex_manager| {
        // println!("tex_set_active_texture {:?}", (canvas_index, tex_shader_index, tex_id));
        unsafe {
            ctx.ActiveTexture(gl::TEXTURE0 + tex_shader_index as u32);
            ctx.BindTexture(gl::TEXTURE_2D, if tex_id < 0 { tex_manager.temp_tex } else { tex_manager.tex_map[&tex_id] });
        }
    });
}
pub fn tex_draw_end(canvas_index: i32, draw_count: i32) {
    paint!(canvas_index, move |ctx, tex_manager| {
        // println!("tex_draw_end {:?}", (canvas_index, draw_count));
        unsafe {
            let tex_pos_buf = &mut *tex_manager.tex_pos_buf;
            let draw_pos_buf = &mut *tex_manager.draw_pos_buf;
            let tex_index_buf = &mut *tex_manager.tex_index_buf;
            ctx.BindBuffer(gl::ARRAY_BUFFER, tex_manager.tex_pos_gl_buf);
            ctx.BufferData(gl::ARRAY_BUFFER, draw_count as isize * 8 * mem::size_of::<f32>() as isize, tex_pos_buf.as_ptr() as *const c_void, gl::STREAM_DRAW);
            ctx.BindBuffer(gl::ARRAY_BUFFER, tex_manager.draw_pos_gl_buf);
            ctx.BufferData(gl::ARRAY_BUFFER, draw_count as isize * 8 * mem::size_of::<f32>() as isize, draw_pos_buf.as_ptr() as *const c_void, gl::STREAM_DRAW);
            ctx.BindBuffer(gl::ARRAY_BUFFER, tex_manager.tex_index_gl_buf);
            ctx.BufferData(gl::ARRAY_BUFFER, draw_count as isize * 4 * mem::size_of::<f32>() as isize, tex_index_buf.as_ptr() as *const c_void, gl::STREAM_DRAW);
            ctx.DrawElements(gl::TRIANGLES, draw_count * 6, gl::UNSIGNED_SHORT, 0 as *const c_void);
        }
    });
}
pub fn tex_set_draw_state(canvas_index: i32, color_r: f32, color_g: f32, color_b: f32, color_a: f32, alpha: f32) {
    paint!(canvas_index, move |ctx, tex_manager| {
        // println!("tex_set_draw_state {:?}", (canvas_index, color_r, color_g, color_b, color_a, alpha));
        unsafe {
            ctx.Uniform4f(tex_manager.u_color, color_r, color_g, color_b, color_a);
            ctx.Uniform1f(tex_manager.u_alpha, alpha);
        }
    });
}
