#![allow(dead_code)]

use std::{f64::consts::PI, fmt, sync::Arc};

use glam::DVec3;

use crate::{
    math::{Bounds3, DInterval, Ray},
    render::{HitRecord, Hittable, Material},
};

#[derive(Clone)]
pub struct Sphere {
    center: DVec3,
    radius: f64,
    mat: Arc<dyn Material>,
    aabb: Bounds3,
}

impl fmt::Debug for Sphere {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sphere")
            .field("center", &self.center)
            .field("radius", &self.radius)
            .finish()
    }
}

impl Sphere {
    pub fn new(center: DVec3, radius: f64, mat: Arc<dyn Material>) -> Self {
        let aabb = Bounds3::new(center - radius, center + radius);
        Self {
            center,
            radius: f64::max(0.0, radius),
            mat,
            aabb,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, ray_t: DInterval) -> Option<HitRecord> {
        let oc = self.center - ray.origin();

        let a = ray.direction().length_squared();
        let h = ray.direction().dot(oc);
        let c = oc.length_squared() - self.radius.powi(2);
        let discriminant = h.powi(2) - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();
        let mut root = (h - sqrtd) / a;

        // find nearest root in the acceptable range
        if !ray_t.surrounds(root) {
            root = (h + sqrtd) / a;
            if !ray_t.surrounds(root) {
                return None;
            }
        }

        let mut rec = HitRecord::default();
        rec.t = root;
        rec.point = ray.at(rec.t);
        let d = (rec.point - self.center).normalize();
        rec.u = 0.5 + (-d.z).atan2(d.x) / (2.0 * PI);
        rec.v = 0.5 + d.y.asin() / (PI);
        let outward_normal = (rec.point - self.center) / self.radius;
        rec.set_face_normal(ray, outward_normal);
        rec.mat = self.mat.clone();

        Some(rec)
    }

    fn aabb(&self) -> Bounds3 {
        self.aabb
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use crate::render::null_material_ptr;

    use super::*;

    /*
    #[test]
    fn test_hit_happy_path() {
        let x_loc = 0.0;
        let y_loc = 0.0;
        let z_loc = -1.0;
        let rad = 0.5;

        let s = Sphere::new(DVec3::new(x_loc, y_loc, z_loc), rad, null_material_ptr());

        let ray = Ray::new(DVec3::ZERO, DVec3::new(0.0, 0.0, -1.0));
        let ray_t = DInterval::UNIVERSE;

        let ray_hit = s.hit(&ray, ray_t).unwrap();
        assert_relative_eq!(ray_hit.t, 0.5);
        assert_eq!(ray_hit.point, DVec3::new(0.0, 0.0, -0.5));
        assert_eq!(ray_hit.normal, DVec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_hit_exactly_tangent() {
        let x_loc = 0.0;
        let y_loc = 0.0;
        let z_loc = 0.0;
        let rad = 0.5;

        let s = Sphere::new(DVec3::new(x_loc, y_loc, z_loc), rad, null_material_ptr());

        let ray = Ray::new(DVec3::new(0.0, 0.5, 1.0), DVec3::new(0.0, 0.0, -1.0));
        let ray_t = DInterval::UNIVERSE;

        let ray_hit = s.hit(&ray, ray_t).unwrap();
        assert_relative_eq!(ray_hit.t, 1.0);
        assert_eq!(ray_hit.point, DVec3::new(0.0, 0.5, 0.0));
        // NOTE: in the case of a perfectly tangent hit, the "is outward facing" part of normal
        // calculation will behave inconsistently, since normal.dot(ray) will always be zero,
        // regardless of the location on the sphere. I'm choosing to only test absolute normal here.
        assert_eq!(ray_hit.normal.abs(), DVec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn test_aabb() {
        let x_loc = 1.0;
        let y_loc = 0.0;
        let z_loc = -1.0;
        let rad = 0.5;

        let s = Sphere::new(DVec3::new(x_loc, y_loc, z_loc), rad, null_material_ptr());

        let expected_min = DVec3::new(x_loc - rad, y_loc - rad, z_loc - rad);
        let expected_max = DVec3::new(x_loc + rad, y_loc + rad, z_loc + rad);
        assert_eq!(s.aabb(), Bounds3::new(expected_min, expected_max));
    }
    */
}
