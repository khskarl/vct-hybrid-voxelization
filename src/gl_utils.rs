use gl_helpers::*;

use std::ffi::CStr;
use std::str;
use std::{fs::File, io::{BufWriter, Write}};

pub fn print_opengl_diagnostics() {
	let write_file = File::create("./diagnostics.log").unwrap();
	let mut writer = BufWriter::new(&write_file);
	let gl_info = GLInfo::new();

	writeln!(&mut writer, "OpenGL version string : {}", gl_info.version());

	writeln!(&mut writer, 
		"OpenGL version : {:?}.{:?}",
		gl_info.major(),
		gl_info.minor(),
	);

	writeln!(&mut writer, 
		"GLSL version : {:?}.{:?}0",
		gl_info.glsl_major(),
		gl_info.glsl_minor()
	);

	let mut max_geometry_tex = 0;
	let mut group_size = [0, 0, 0];
	let mut max_atomic_counter_bindings = 0;
	let mut max_atomic_counter_buffer_size = 66;
	unsafe {
		gl::GetIntegerv(gl::MAX_GEOMETRY_TEXTURE_IMAGE_UNITS, &mut max_geometry_tex);
		gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_SIZE, 0, &mut group_size[0]);
		gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_SIZE, 1, &mut group_size[1]);
		gl::GetIntegeri_v(gl::MAX_COMPUTE_WORK_GROUP_SIZE, 2, &mut group_size[2]);
		gl::GetIntegerv(
			gl::MAX_ATOMIC_COUNTER_BUFFER_BINDINGS,
			&mut max_atomic_counter_bindings,
		);
		gl::GetIntegerv(
			gl::ATOMIC_COUNTER_BUFFER_SIZE,
			&mut max_atomic_counter_buffer_size,
		);
	}
	writeln!(&mut writer, "MAX_GEOMETRY_TEXTURE_IMAGE_UNITS : {}", max_geometry_tex);
	writeln!(&mut writer, 
		"MAX_COMPUTE_WORK_GROUP_SIZE : ({}, {}, {})",
		group_size[0], group_size[1], group_size[2]
	);
	writeln!(&mut writer, 
		"MAX_ATOMIC_COUNTER_BUFFER_BINDINGS : {}",
		max_atomic_counter_bindings
	);

	writeln!(&mut writer, 
		"ATOMIC_COUNTER_BUFFER_SIZE : {}",
		max_atomic_counter_buffer_size
	);
	
	let i_need_these_extensions_please = vec![
		"GL_EXT_texture3D",
		"GL_NV_conservative_raster",
		"GL_INTEL_conservative_rasterization",
	];

	writeln!(&mut writer, "EXTENSIONS");
	writeln!(&mut writer, "----------");
	for extension in i_need_these_extensions_please {
		writeln!(&mut writer, "{} : {}", extension, is_extension_supported(extension));
	}

	write_file.sync_all().unwrap();

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
