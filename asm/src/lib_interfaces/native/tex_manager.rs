use std::collections::HashMap;
use std::ptr;
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
}
impl TexManager {
    pub fn new(ctx: &mut Gl, tex_size: i32, tex_count: i32) -> Self {
        unsafe {
            let img_shader_program = create_program(ctx, include_str!("glsl/img.v.glsl"), include_str!("glsl/img.f.glsl"));
            ctx.UseProgram(img_shader_program);

            // create gl buffers
            let mut gl_bufs = [0; 4];
            ctx.GenBuffers(4, gl_bufs.as_mut_ptr());

            // the texture position buffer
            let tex_pos_gl_buf = gl_bufs[0];
            let tex_pos_buf = Box::new([0.; GL_DRAW_RECT_MAX as usize * 8]);
            ctx.BindBuffer(gl::ARRAY_BUFFER, tex_pos_gl_buf);
            ctx.BufferData(gl::ARRAY_BUFFER, GL_DRAW_RECT_MAX as isize * 8, tex_pos_buf.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);
            let a_tex_pos = ctx.GetAttribLocation(img_shader_program, buffer_from_str("aTexPos")) as u32;
            ctx.EnableVertexAttribArray(a_tex_pos);
            ctx.VertexAttribPointer(a_tex_pos, 2, gl::FLOAT, gl::FALSE, 0, 0 as *const c_void);

            // the draw position buffer
            let draw_pos_gl_buf = gl_bufs[1];
            let draw_pos_buf = Box::new([0.; GL_DRAW_RECT_MAX as usize * 8]);
            ctx.BindBuffer(gl::ARRAY_BUFFER, draw_pos_gl_buf);
            ctx.BufferData(gl::ARRAY_BUFFER, GL_DRAW_RECT_MAX as isize * 8, draw_pos_buf.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);
            let a_draw_pos = ctx.GetAttribLocation(img_shader_program, buffer_from_str("aDrawPos")) as u32;
            ctx.EnableVertexAttribArray(a_draw_pos);
            ctx.VertexAttribPointer(a_draw_pos, 2, gl::FLOAT, gl::FALSE, 0, 0 as *const c_void);

            // the draw position buffer
            let tex_index_gl_buf = gl_bufs[2];
            let tex_index_buf = Box::new([0.; GL_DRAW_RECT_MAX as usize * 4]);
            ctx.BindBuffer(gl::ARRAY_BUFFER, tex_index_gl_buf);
            ctx.BufferData(gl::ARRAY_BUFFER, GL_DRAW_RECT_MAX as isize * 4, tex_index_buf.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);
            let a_tex_index = ctx.GetAttribLocation(img_shader_program, buffer_from_str("aTexIndex")) as u32;
            ctx.EnableVertexAttribArray(a_tex_index);
            ctx.VertexAttribPointer(a_tex_index, 1, gl::FLOAT, gl::FALSE, 0, 0 as *const c_void);

            // the element indices buffer
            let index_gl_buf = gl_bufs[3];
            let mut index_buf = Box::new([0 as u16; GL_DRAW_RECT_MAX as usize * 6]);
            generate_index_buf_content(&mut index_buf);
            ctx.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_gl_buf);
            ctx.BufferData(gl::ELEMENT_ARRAY_BUFFER, GL_DRAW_RECT_MAX as isize * 6, index_buf.as_ptr() as *const c_void, gl::STATIC_DRAW);

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
            ctx.Uniform2f(self.u_area_size, w as f32, h as f32);
        }
    }

    pub fn tex_create(&mut self, ctx: &mut Gl, w: i32, h: i32, buf: &Box<[u8]>, tex_id: i32) {
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
            ctx.TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, w, h, 0, gl::RGBA, gl::UNSIGNED_BYTE, (**buf).as_ptr() as *const c_void);
        }
    }
}

pub fn tex_create_empty(canvas_index: i32, tex_id: i32, width: i32, height: i32) {
    paint!(canvas_index, move |ctx, _tex_manager| {
        // TODO
    });
}
pub fn tex_copy(canvas_index: i32, dest_tex_id: i32, dest_left: i32, dest_top: i32, src_left: i32, src_top: i32, width: i32, height: i32) {
    unimplemented!();
}
pub fn tex_bind_rendering_target(canvas_index: i32, texId: i32, width: i32, height: i32) {
    unimplemented!();
}
pub fn tex_unbind_rendering_target(canvas_index: i32) {
    unimplemented!();
}
pub fn tex_delete(canvas_index: i32, texId: i32) {
    unimplemented!();
}
pub fn tex_draw(canvas_index: i32, drawIndex: i32, texShaderIndex: i32, normalizedTexX: f64, normalizedTexY: f64, normalizedTexW: f64, normalizedTexH: f64, x: f64, y: f64, w: f64, h: f64) {
    unimplemented!();
}
pub fn tex_set_active_texture(canvas_index: i32, texShaderIndex: i32, texId: i32) {
    unimplemented!();
}
pub fn tex_draw_end(canvas_index: i32, drawCount: i32) {
    unimplemented!();
}
pub fn tex_set_draw_state(canvas_index: i32, colorR: f32, colorG: f32, colorB: f32, colorA: f32, alpha: f32) {
    unimplemented!();
}
