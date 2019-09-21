use image;
use image::ImageFormat::{JPEG, PNG};

pub struct Model {
	pub positions: Vec<[f32; 3]>,
	pub tex_coords: Vec<[f32; 2]>,
	pub indices: Vec<u32>,
	pub color_texture: image::DynamicImage,
}

impl Model {
	pub fn new(path: &str) -> Model {
		let gltf_document = gltf::import(path);
		let (gltf, buffers, _) = gltf_document.unwrap();

		let mut meshes = Vec::<gltf::mesh::Mesh>::new();

		for mesh in gltf.meshes() {
			println!("[Mesh #{}]", mesh.index());
			meshes.push(mesh);
		}

		let mesh = meshes.first().unwrap();
		let mut positions = Vec::<[f32; 3]>::new();
		let mut tex_coords = Vec::<[f32; 2]>::new();
		let mut indices = Vec::<u32>::new();
		let mut color_texture = image::DynamicImage::new_rgba8(64, 64);

		for primitive in mesh.primitives() {
			let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

			if let Some(iter) = reader.read_positions() {
				for vertex_position in iter {
					positions.push(vertex_position);
				}
			}

			if let Some(iter) = reader.read_tex_coords(0) {
				for tex_coord in iter.into_f32() {
					tex_coords.push(tex_coord);
				}
			}

			if let Some(iter) = reader.read_indices() {
				for index in iter.into_u32() {
					indices.push(index);
				}
			}

			let material = primitive.material();
			let pbr_metallic_roughness = material.pbr_metallic_roughness();
			let base_color_texture = pbr_metallic_roughness
				.base_color_texture()
				.unwrap()
				.texture();

			color_texture = load_gltf_texture(&buffers, base_color_texture);
		}

		println!("# vertices: {}", positions.len());
		println!("# indices: {}", indices.len());

		Model {
			positions,
			tex_coords,
			indices,
			color_texture,
		}
	}

	pub fn info(path: &str) {
		let (gltf, _buffers, _images) = gltf::import(path).unwrap();

		for mesh in gltf.meshes() {
			println!("[Mesh #{}]", mesh.index());

			for primitive in mesh.primitives() {
				let material = primitive.material();
				let material_name = match material.index() {
					Some(index) => index.to_string(),
					None => "default".to_string(),
				};

				println!(
					"  Primitive {} with Mat {}",
					primitive.index(),
					material_name,
				);
			}
		}

		for material in gltf.materials() {
			println!("[Material #{}]", material.index().unwrap());
			println!("  Alpha Cutoff: {:?}", material.alpha_cutoff());
			println!("  Alpha Mode: {:?}", material.alpha_mode());
		}
	}
}

fn load_gltf_texture(
	buffers: &Vec<gltf::buffer::Data>,
	texture: gltf::Texture<'_>,
) -> image::DynamicImage {
	use gltf::image::*;

	let img = match texture.source().source() {
		Source::View { view, mime_type } => {
			// use log::*;
			// info!("Loading image: {:?}", texture.source().name());
			let buffer_data = &buffers[view.buffer().index()].0;
			let begin = view.offset();
			let end = begin + view.length();
			let data = &buffer_data[begin..end];

			match mime_type {
				"image/jpeg" => image::load_from_memory_with_format(data, JPEG),
				"image/png" => image::load_from_memory_with_format(data, PNG),
				_ => panic!(format!("unsupported image type (mime_type: {})", mime_type)),
			}
		}
		gltf::image::Source::Uri { .. } => {
			unimplemented!();
		}
	};

	let dyn_img = img.expect("Failed to load image to CPU memory");

	dyn_img
}
