use gl::types::*;
use gl_helpers::*;

use std::ffi::CStr;
use std::str;

// Shader sources
pub static VS_SRC: &'static str = "
    #version 330

    uniform mat4 proj;
    uniform mat4 view;
    uniform sampler2D albedo;

    layout (location = 0) in vec3 aPosition;
    layout (location = 1) in vec2 aTexCoord;
    layout (location = 2) in vec3 aNormal;

    out vec2 v_uv;
    out vec3 v_normal;

    void main() {
        gl_Position = (proj * view) * vec4(aPosition, 1.0);
        v_uv = aTexCoord;
        v_normal = aNormal;
    }
";

pub static FS_SRC: &'static str = "
    #version 330

    vec3 light_dir = vec3(0.3, 1.0, 0.4);

    uniform mat4 proj;
    uniform mat4 view;
    uniform float time;
    uniform sampler2D albedo;

    in vec2 v_uv;
    in vec3 v_normal;

    out vec4 out_color;

    void main() {
        vec2 uv = vec2(v_uv.x + sin(time) * 0.001, v_uv.y);
        vec3 diffuse = texture2D(albedo, uv).xyz;
        vec3 color = vec3(0.0, 0.0, 0.0);
        color += diffuse * vec3(0.8, 0.8, 0.7) * dot(normalize(light_dir), v_normal);
        color += diffuse * vec3(0.3, 0.35, 0.25);
        out_color = vec4(color, 1.0);
    }
";

pub fn print_opengl_diagnostics() {
	let gl_info = GLInfo::new();
	println!("OpenGL version string : {}", gl_info.version());

	println!(
		"OpenGL version : {:?}.{:?}",
		gl_info.major(),
		gl_info.minor(),
	);

	println!(
		"GLSL version : {:?}.{:?}0",
		gl_info.glsl_major(),
		gl_info.glsl_minor()
	);

	print_needed_extensions();
}

pub fn print_needed_extensions() {
	let i_need_these_extensions_please = vec![
		"GL_EXT_texture3D",
		"GL_NV_conservative_raster",
		"GL_INTEL_conservative_rasterization",
	];

	println!("EXTENSIONS");
	println!("----------");
	for extension in i_need_these_extensions_please {
		println!("{} : {}", extension, is_extension_supported(extension));
	}
}

#[allow(dead_code)]
pub fn print_extensions() {
	let mut num_extensions = 0;
	unsafe {
		gl::GetIntegerv(gl::NUM_EXTENSIONS, &mut num_extensions);
	}

	println!("Num of extensions: {}", num_extensions);

	for i in 0..num_extensions {
		unsafe {
			let extension_name = gl::GetStringi(gl::EXTENSIONS, i as u32) as *const i8;
			let extension_name = CStr::from_ptr(extension_name);
			println!("Extension {}: {:?}", i, extension_name);
		}
	}
}

pub fn is_extension_supported(extension: &str) -> bool {
	let mut num_extensions = 0;
	unsafe {
		gl::GetIntegerv(gl::NUM_EXTENSIONS, &mut num_extensions);
	}

	for i in 0..num_extensions {
		unsafe {
			let extension_name = gl::GetStringi(gl::EXTENSIONS, i as u32) as *const i8;
			let extension_name = CStr::from_ptr(extension_name).to_str().unwrap();

			if extension_name == extension {
				return true;
			}
		}
	}

	return false;
}
