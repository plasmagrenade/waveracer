use std::sync::Arc;

use crate::{
    math::{
        Color, Ray, near_zero,
        random::{random_double, random_unit_vector},
    },
    render::{HitRecord, Material, ScatterData, SolidColor, Texture},
};
use glam::DVec3;

#[derive(Default, Copy, Clone, Debug)]
pub struct Dielectric {
    pub refraction_index: f64,
}

impl Dielectric {
    pub fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
    }

    /// Approximate Fresnel contribution for reflection
    // TODO: is "ratio" the right term here?
    fn schlick_approx(&self, cosine: f64, refraction_ratio: f64) -> f64 {
        let mut r0 = (1.0 - refraction_ratio) / (1.0 + refraction_ratio);
        r0 *= r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord) -> Option<ScatterData> {
        let attenuation = Color::ONE;
        let refraction_ratio = if rec.is_front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = ray_in.direction().normalize();
        let cos_theta = (-unit_direction).dot(rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();
        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let direction = if cannot_refract
            || self.schlick_approx(cos_theta, refraction_ratio) > random_double()
        {
            // must reflect
            ray_in.direction().reflect(rec.normal)
        } else {
            // can refract
            unit_direction.refract(rec.normal, refraction_ratio)
        };

        let result = ScatterData {
            attenuation,
            scattered: Ray::new(rec.point, direction, ray_in.pixel_x, ray_in.pixel_y),
        };
        Some(result)
    }
}
