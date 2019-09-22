use image;
use image::ImageFormat::{JPEG, PNG};

pub struct Mesh {
	primitives: Vec<Primitive>,
}

impl Mesh {
	pub fn new(path: &str) -> Mesh {
		let gltf_document = gltf::import(path);
		let (gltf, buffers, _) = gltf_document.unwrap();

		let mut primitives = Vec::<Primitive>::new();

		for gltf_mesh in gltf.meshes() {
			for gltf_primitive in gltf_mesh.primitives() {
				primitives.push(Primitive::new(&buffers, &gltf_primitive));
			}
		}

		Mesh { primitives }
	}

	pub fn primitives(&self) -> &Vec<Primitive> {
		&self.primitives
	}
}

pub struct Primitive {
	pub positions: Vec<[f32; 3]>,
	pub tex_coords: Vec<[f32; 2]>,
	pub normals: Vec<[f32; 3]>,
	pub indices: Vec<u32>,
	pub color_texture: image::DynamicImage,
}

impl Primitive {
	pub fn new(buffers: &Vec<gltf::buffer::Data>, gltf_primitive: &gltf::Primitive) -> Primitive {
		let mut positions = Vec::<[f32; 3]>::new();
		let mut tex_coords = Vec::<[f32; 2]>::new();
		let mut normals = Vec::<[f32; 3]>::new();
		let mut indices = Vec::<u32>::new();

		let reader = gltf_primitive.reader(|buffer| Some(&buffers[buffer.index()]));

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

		if let Some(iter) = reader.read_normals() {
			for normal in iter {
				normals.push(normal);
			}
		}

		if let Some(iter) = reader.read_indices() {
			for index in iter.into_u32() {
				indices.push(index);
			}
		}

		let material = load_gltf_material(&buffers, gltf_primitive.material());
		// println!("# vertices: {}", positions.len());
		// println!("# indices: {}", indices.len());

		Primitive {
			positions,
			tex_coords,
			normals,
			indices,
			material,
		}
	}

	#[allow(dead_code)]
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

fn load_gltf_material(buffers: &Vec<gltf::buffer::Data>, material: gltf::Material<'_>) {
	let pbr_metallic_roughness = material.pbr_metallic_roughness();
	let base_color_texture = pbr_metallic_roughness
		.base_color_texture()
		.unwrap()
		.texture();

	let metallic_roughness_texture = pbr_metallic_roughness
		.metallic_roughness_texture()
		.unwrap()
		.texture();

	let normal_texture = material.normal_texture().unwrap().texture();

	let color_texture_image = load_gltf_texture(&buffers, base_color_texture);
	let metallic_roughness_texture_image = load_gltf_texture(&buffers, metallic_roughness_texture)

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
