use image::{DynamicImage, ImageBuffer};

use std::rc::Rc;

pub struct MaterialBuilder {
	name: String,
	albedo_tex: Option<Rc<Texture>>,
	normal_tex: Option<Rc<Texture>>,
	metaghness_tex: Option<Rc<Texture>>,
	occlusion_tex: Option<Rc<Texture>>,
}

impl MaterialBuilder {
	pub fn new(name: String) -> MaterialBuilder {
		MaterialBuilder {
			name,
			albedo_tex: None,
			normal_tex: None,
			metaghness_tex: None,
			occlusion_tex: None,
		}
	}

	pub fn albedo_tex(mut self, image: Rc<Texture>) -> MaterialBuilder {
		self.albedo_tex = Some(image);
		self
	}

	pub fn metaghness_tex(mut self, image: Rc<Texture>) -> MaterialBuilder {
		self.metaghness_tex = Some(image);
		self
	}

	pub fn normal_tex(mut self, image: Rc<Texture>) -> MaterialBuilder {
		self.normal_tex = Some(image);
		self
	}

	pub fn occlusion_tex(mut self, image: Rc<Texture>) -> MaterialBuilder {
		self.occlusion_tex = Some(image);
		self
	}

	pub fn build(self) -> Material {
		let albedo = self.albedo_tex.unwrap_or(Rc::new(default_albedo()));
		let metaghness = self.metaghness_tex.unwrap_or(Rc::new(default_metaghness()));
		let normal = self.normal_tex.unwrap_or(Rc::new(default_normal()));
		let occlusion = self.occlusion_tex.unwrap_or(Rc::new(default_occlusion()));

		Material::new(self.name, albedo, metaghness, normal, occlusion)
	}
}

pub struct Material {
	name: String,
	albedo: Rc<Texture>,
	metaghness: Rc<Texture>,
	normal: Rc<Texture>,
	occlusion: Rc<Texture>,
}

impl Material {
	pub fn new(
		name: String,
		albedo: Rc<Texture>,
		metaghness: Rc<Texture>,
		normal: Rc<Texture>,
		occlusion: Rc<Texture>,
	) -> Material {
		Material {
			name,
			albedo,
			metaghness,
			normal,
			occlusion,
		}
	}

	pub fn name(&self) -> &String {
		&self.name
	}
	pub fn albedo(&self) -> &Texture {
		&self.albedo
	}
	pub fn metaghness(&self) -> &Texture {
		&self.metaghness
	}
	pub fn normal(&self) -> &Texture {
		&self.normal
	}
	pub fn occlusion(&self) -> &Texture {
		&self.occlusion
	}
}

fn default_albedo() -> Texture {
	let img = ImageBuffer::from_fn(1, 1, |x, y| {
		if x % 2 == 0 {
			image::Rgb([0u8, 0u8, 0u8])
		} else {
			image::Rgb([255u8, 255u8, 255u8])
		}
	});

	Texture::new("default_albedo".to_owned(), DynamicImage::ImageRgb8(img))
}

fn default_metaghness() -> Texture {
	let img = ImageBuffer::from_fn(256, 256, |_, _| image::Rgb([0u8, 250u8, 0u8]));

	Texture::new(
		"default_metaghness".to_owned(),
		DynamicImage::ImageRgb8(img),
	)
}

fn default_normal() -> Texture {
	let img = ImageBuffer::from_fn(1, 1, |_, _| image::Rgb([0, 0, 255u8]));

	Texture::new("default_normal".to_owned(), DynamicImage::ImageRgb8(img))
}

fn default_occlusion() -> Texture {
	let img = ImageBuffer::from_fn(1, 1, |_, _| image::Luma([255u8]));

	Texture::new(
		"default_occlusion".to_owned(),
		DynamicImage::ImageLuma8(img),
	)
}

pub struct Texture {
	name: String,
	image: DynamicImage,
}

impl Texture {
	pub fn new(name: String, image: DynamicImage) -> Texture {
		Texture { name, image }
	}

	pub fn name(&self) -> &String {
		&self.name
	}

	pub fn image(&self) -> &DynamicImage {
		&self.image
	}
}
