use image::{DynamicImage, ImageBuffer};

use std::rc::Rc;

pub struct MaterialBuilder {
	albedo_tex: Option<Rc<DynamicImage>>,
	normal_tex: Option<Rc<DynamicImage>>,
	metaghness_tex: Option<Rc<DynamicImage>>,
	occlusion_tex: Option<Rc<DynamicImage>>,
}

impl MaterialBuilder {
	pub fn new() -> MaterialBuilder {
		MaterialBuilder {
			albedo_tex: None,
			normal_tex: None,
			metaghness_tex: None,
			occlusion_tex: None,
		}
	}

	pub fn albedo_tex(mut self, image: Rc<DynamicImage>) -> MaterialBuilder {
		self.albedo_tex = Some(image);
		self
	}

	pub fn metaghness_tex(mut self, image: Rc<DynamicImage>) -> MaterialBuilder {
		self.metaghness_tex = Some(image);
		self
	}

	pub fn normal_tex(mut self, image: Rc<DynamicImage>) -> MaterialBuilder {
		self.normal_tex = Some(image);
		self
	}

	pub fn occlusion_tex(mut self, image: Rc<DynamicImage>) -> MaterialBuilder {
		self.occlusion_tex = Some(image);
		self
	}

	pub fn build(self) -> Material {
		let albedo = self.albedo_tex.unwrap_or(Rc::new(default_albedo()));
		let metaghness = self.metaghness_tex.unwrap_or(Rc::new(default_metaghness()));
		let normal = self.normal_tex.unwrap_or(Rc::new(default_normal()));
		let occlusion = self.occlusion_tex.unwrap_or(Rc::new(default_occlusion()));

		Material::new(albedo, metaghness, normal, occlusion)
	}
}

pub struct Material {
	albedo: Rc<DynamicImage>,
	metaghness: Rc<DynamicImage>,
	normal: Rc<DynamicImage>,
	occlusion: Rc<DynamicImage>,
}

impl Material {
	pub fn new(
		albedo: Rc<DynamicImage>,
		metaghness: Rc<DynamicImage>,
		normal: Rc<DynamicImage>,
		occlusion: Rc<DynamicImage>,
	) -> Material {
		Material {
			albedo,
			metaghness,
			normal,
			occlusion,
		}
	}

	pub fn albedo(&self) -> &DynamicImage {
		&self.albedo
	}
	pub fn metaghness(&self) -> &DynamicImage {
		&self.metaghness
	}
	pub fn normal(&self) -> &DynamicImage {
		&self.normal
	}
	pub fn occlusion(&self) -> &DynamicImage {
		&self.occlusion
	}
}

fn default_albedo() -> DynamicImage {
	let img = ImageBuffer::from_fn(1, 1, |x, y| {
		if x % 2 == 0 {
			image::Rgb([0u8, 0u8, 0u8])
		} else {
			image::Rgb([255u8, 255u8, 255u8])
		}
	});
	DynamicImage::ImageRgb8(img)
}

fn default_metaghness() -> DynamicImage {
	let img = ImageBuffer::from_fn(256, 256, |x, y| {
		if x % 2 == 0 {
			image::Rgb([0u8, 0u8, 0u8])
		} else {
			image::Rgb([255u8, 255u8, 255u8])
		}
	});

	DynamicImage::ImageRgb8(img)
}

fn default_normal() -> DynamicImage {
	let img = ImageBuffer::from_fn(1, 1, |x, y| image::Rgb([0, 0, 255u8]));
	DynamicImage::ImageRgb8(img)
}

fn default_occlusion() -> DynamicImage {
	let img = ImageBuffer::from_fn(1, 1, |x, y| image::Luma([255u8]));
	DynamicImage::ImageLuma8(img)
}
