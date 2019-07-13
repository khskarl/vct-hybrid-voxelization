// extern crate nalgebra_glm as glm;
use nalgebra_glm as glm;

pub struct Camera {
  pub yaw: f32,
  pub pitch: f32,
  pub proj: glm::Mat4,
}
