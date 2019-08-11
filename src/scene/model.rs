use gltf::Gltf;

#[derive(Debug)]
pub struct Model<'a> {
		pub positions: Vec<[f32; 3]>,
		pub tex_coords: Vec<[f32; 2]>,
		pub indices: Vec<u32>,
		pub texture: gltf::Texture<'a>,
}

impl<'a> Model<'a> {
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
		let mut texture;
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

			let material = primitive.material().clone();
			let pbr_metallic_roughness = material.pbr_metallic_roughness();
			texture = pbr_metallic_roughness.base_color_texture().unwrap().texture().clone();
			// println!("Material: {}", pbr_metallic_roughness.index().unwrap());
		}

		Model {
			positions,
			tex_coords,
			indices,
			texture,
		}
	}
}
