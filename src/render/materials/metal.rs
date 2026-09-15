use crate::{
    math::{Color, Ray, random::random_unit_vector},
    render::{HitRecord, Material, ScatterData},
};

#[derive(Default, Copy, Clone, Debug)]
pub struct Metal {
    pub albedo: Color,
    pub fuzziness: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzziness: f64) -> Self {
        Self {
            albedo,
            fuzziness: fuzziness.clamp(0.0, 1.0),
        }
    }
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord) -> Option<ScatterData> {
        // perfect reflectance - ray gets reflected about the normal
        let mut reflected = ray_in.direction().reflect(rec.normal);
        reflected = reflected.normalize() + (self.fuzziness * random_unit_vector());

        let result = ScatterData {
            attenuation: self.albedo,
            scattered: Ray::new(rec.point, reflected, ray_in.pixel_x, ray_in.pixel_y),
        };
        if result.scattered.direction().dot(rec.normal) > 0.0 {
            Some(result)
        } else {
            None
        }
    }
}
