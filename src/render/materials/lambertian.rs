use std::{fmt, sync::Arc};

use crate::{
    math::{Color, Ray, near_zero, random::random_unit_vector},
    render::{HitRecord, Material, ScatterData, SolidColor, Texture},
};

#[derive(Clone)]
pub struct Lambertian {
    tex: Arc<dyn Texture>,
}

impl fmt::Debug for Lambertian {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Lambertian").finish()
    }
}

impl Lambertian {
    pub fn new(tex: Arc<dyn Texture>) -> Self {
        Self { tex }
    }

    pub fn from_color(albedo: Color) -> Self {
        Self {
            tex: Arc::new(SolidColor::new(albedo)),
        }
    }
}

impl Material for Lambertian {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord) -> Option<ScatterData> {
        // diffuse reflectance - ray gets scattered in a random dir from the normal
        let mut scatter_dir = rec.normal + random_unit_vector();
        if near_zero(scatter_dir) {
            scatter_dir = rec.normal
        }

        let result = ScatterData {
            attenuation: self.tex.value(rec.u, rec.v, rec.point),
            scattered: Ray::new(rec.point, scatter_dir, ray_in.pixel_x, ray_in.pixel_y),
        };
        Some(result)
    }
}
