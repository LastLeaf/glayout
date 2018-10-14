extern crate gl_generator;

use gl_generator::{Registry, Api, Profile, Fallbacks, StructGenerator};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&dest).join("bindings.rs")).unwrap();

    let gles_registry = Registry::new(
        Api::Gles2,
        (3, 2),
        Profile::Compatibility,
        Fallbacks::None,
        vec![
            "GL_ANGLE_framebuffer_multisample",
            "GL_APPLE_framebuffer_multisample",
            "GL_APPLE_sync",
            "GL_ARM_rgba8",
            "GL_EXT_buffer_storage",
            "GL_EXT_disjoint_timer_query",
            "GL_EXT_multi_draw_indirect",
            "GL_EXT_multisampled_render_to_texture",
            "GL_EXT_occlusion_query_boolean",
            "GL_EXT_primitive_bounding_box",
            "GL_EXT_robustness",
            "GL_KHR_debug",
            "GL_NV_copy_buffer",
            "GL_NV_framebuffer_multisample",
            "GL_NV_internalformat_sample_query",
            "GL_NV_pixel_buffer_object",
            "GL_OES_depth_texture",
            "GL_OES_draw_elements_base_vertex",
            "GL_OES_packed_depth_stencil",
            "GL_OES_primitive_bounding_box",
            "GL_OES_rgb8_rgba8",
            "GL_OES_texture_buffer",
            "GL_OES_texture_npot",
            "GL_OES_vertex_array_object",
            "GL_OES_vertex_type_10_10_10_2",
        ],
    );

    gles_registry
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
}
