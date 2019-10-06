use crate::scene::material::{Material, MaterialBuilder, Texture};
use image::ImageFormat::{JPEG, PNG};
use nalgebra_glm as glm;
use std::collections::HashMap;
use std::rc::Rc;

use img_hash::{HashType, ImageHash};
extern crate img_hash;

pub struct Resources {
	textures: HashMap<String, Rc<Texture>>,
	materials: HashMap<String, Rc<Material>>,
}

impl Resources {
	pub fn new() -> Resources {
		Resources {
			textures: HashMap::new(),
			materials: HashMap::new(),
		}
	}
}

pub struct Mesh {
	primitives: Vec<Primitive>,
	pub position: glm::Vec3,
	pub scale: glm::Vec3,
}

impl Mesh {
	pub fn new(path: &str, position: glm::Vec3, scale: glm::Vec3, resources: &mut Resources) -> Mesh {
		let gltf_document = gltf::import(path);
		let (gltf, buffers, _) = gltf_document.unwrap();

		let mut primitives = Vec::<Primitive>::new();

		for gltf_mesh in gltf.meshes() {
			for gltf_primitive in gltf_mesh.primitives() {
				let primitive = Primitive::new(&buffers, &gltf_primitive, resources);
				primitives.push(primitive);
			}
		}

		Mesh {
			primitives,
			position,
			scale,
		}
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
	pub material: Rc<Material>,
}

impl Primitive {
	pub fn new(
		buffers: &Vec<gltf::buffer::Data>,
		gltf_primitive: &gltf::Primitive,
		resources: &mut Resources,
	) -> Primitive {
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

		let material = fetch_gltf_material(&buffers, gltf_primitive.material(), resources);
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

fn fetch_gltf_material(
	buffers: &Vec<gltf::buffer::Data>,
	material: gltf::Material<'_>,
	resources: &mut Resources,
) -> Rc<Material> {
	let key = material
		.name()
		.expect("We don't support unnamed materials :(")
		.to_owned();

	if let Some(material_rc) = resources.materials.get(&key) {
		println!("Fetching material '{}'...", key);
		Rc::clone(material_rc)
	} else {
		println!("Loading material '{}'...", key);

		let material_rc = Rc::new(load_gltf_material(&buffers, material, resources));
		resources
			.materials
			.insert(key.to_owned(), Rc::clone(&material_rc));

		Rc::clone(&material_rc)
	}
}

fn load_gltf_material(
	buffers: &Vec<gltf::buffer::Data>,
	material: gltf::Material<'_>,
	resources: &mut Resources,
) -> Material {
	let mut material_builder = MaterialBuilder::new(material.name().expect("PLS WORK").to_owned());
	let pbr_metallic_roughness = material.pbr_metallic_roughness();

	let base_color_texture = pbr_metallic_roughness.base_color_texture();
	let metallic_roughness_texture = pbr_metallic_roughness.metallic_roughness_texture();
	let normal_texture = material.normal_texture();
	let occlusion_texture = material.occlusion_texture();

	if let Some(base_color_texture) = base_color_texture {
		let texture = fetch_gltf_texture(buffers, base_color_texture.texture(), resources);
		material_builder = material_builder.albedo_tex(texture);
	}

	if let Some(metaghness) = metallic_roughness_texture {
		let texture = fetch_gltf_texture(buffers, metaghness.texture(), resources);
		material_builder = material_builder.metaghness_tex(texture);
	}

	if let Some(normal_texture) = normal_texture {
		let texture = fetch_gltf_texture(buffers, normal_texture.texture(), resources);
		material_builder = material_builder.normal_tex(texture);
	}

	if let Some(occlusion_texture) = occlusion_texture {
		let texture = fetch_gltf_texture(buffers, occlusion_texture.texture(), resources);
		material_builder = material_builder.occlusion_tex(texture);
	}

	material_builder.build()
}

fn fetch_gltf_texture(
	buffers: &Vec<gltf::buffer::Data>,
	texture: gltf::Texture<'_>,
	resources: &mut Resources,
) -> Rc<Texture> {
	if texture.name().is_some() {
		let key = texture.name().unwrap();

		if let Some(texture_rc) = resources.textures.get(key) {
			// println!("Fetching texture '{}'...", key);
			return Rc::clone(texture_rc);
		}
	}

	let texture = load_gltf_texture(&buffers, texture);
	let key = ImageHash::hash(texture.image(), 8, HashType::Gradient).to_base64();
	if let Some(texture_rc) = resources.textures.get(&key) {
		// println!("Fetching texture '{}'...", key);
		return Rc::clone(texture_rc);
	}

	// println!("Loading texture '{}'...", key);

	let texture_rc = Rc::new(texture);
	resources.textures.insert(key, Rc::clone(&texture_rc));

	Rc::clone(&texture_rc)
}

fn load_gltf_texture(buffers: &Vec<gltf::buffer::Data>, texture: gltf::Texture<'_>) -> Texture {
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

	let hash = ImageHash::hash(&dyn_img, 8, HashType::Gradient);
	Texture::new(hash.to_base64(), dyn_img)
}
