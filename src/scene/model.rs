use gltf::Gltf;

#[derive(Debug)]
pub struct Model {
		pub positions: Vec<[f32; 3]>,
		pub tex_coords: Vec<[f32; 2]>,
		pub indices: Vec<u32>,
}

impl Model {
	pub fn new(path: &str) -> Model {
		let (gltf, buffers, _) = gltf::import(path).unwrap();

		let mut meshes = Vec::<gltf::mesh::Mesh>::new();

		for mesh in gltf.meshes() {
			println!("[Mesh #{}]", mesh.index());
			meshes.push(mesh);
		}

		let mesh = meshes.first().unwrap();
		let mut positions = Vec::<[f32; 3]>::new();
		let mut tex_coords = Vec::<[f32; 2]>::new();
		let mut indices = Vec::<u32>::new();

		for primitive in mesh.primitives() {
			let reader = primitive.reader( |buffer|
				Some(&buffers[buffer.index()])
			);

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

			// let material = primitive.material().clone();
			// let pbr_metallic_roughness = material.pbr_metallic_roughness();
			// texture = pbr_metallic_roughness.base_color_texture().unwrap().texture().clone();
			// println!("Material: {}", pbr_metallic_roughness.index().unwrap());
		}

		Model {
			positions,
			tex_coords,
			indices,
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

				println!("  Primitive {} with Mat {}",
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
