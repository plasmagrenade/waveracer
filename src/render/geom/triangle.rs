use std::{fmt, sync::Arc};

use glam::DVec3;

use crate::{
    math::{ALMOST_ZERO, Bounds3, DInterval, Ray},
    render::{HitRecord, Hittable, Material},
};

#[derive(Clone)]
pub struct Triangle {
    pub a: DVec3,
    pub b: DVec3,
    pub c: DVec3,
    mat: Arc<dyn Material>,
    aabb: Bounds3,
}

impl fmt::Debug for Triangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Triangle")
            .field("a", &self.a)
            .field("b", &self.b)
            .field("c", &self.c)
            .finish()
    }
}

impl Triangle {
    pub fn new(a: DVec3, b: DVec3, c: DVec3, mat: Arc<dyn Material>) -> Self {
        let pts = [a, b, c];
        let aabb = Bounds3::new(
            pts.iter().fold(DVec3::MAX, |cur_min, &pt| cur_min.min(pt)),
            pts.iter().fold(DVec3::MIN, |cur_max, &pt| cur_max.max(pt)),
        );
        Self { a, b, c, mat, aabb }
    }
}

impl Hittable for Triangle {
    fn hit(&self, ray: &Ray, ray_t: DInterval) -> Option<HitRecord> {
        if !self.aabb().intersected_by(ray, ray_t) {
            return None;
        }

        let ab = self.b - self.a;
        let ac = self.c - self.a;

        // NOTE: we don't normalize this because we're using the length squared as a shortcut for
        // "area of the parallelogram defined by AB and AC"
        let normal = ab.cross(ac);

        // find ray intersection with plane
        let t = {
            // NOTE: this is reversed from plane.rs, since ray.direction is unit length but normal isn't
            let denom = normal.dot(ray.direction());

            // ray is parallel to the triangle - ray projected onto normal approaches zero
            if denom.abs() < ALMOST_ZERO {
                return None;
            }

            normal.dot(self.a - ray.origin()) / denom
        };

        if !ray_t.surrounds(t) {
            return None;
        }

        let p = ray.at(t);

        // TODO: grok and document this
        let twice_abc_area = normal.length_squared();
        let twice_bcp_area = (self.c - self.b).cross(p - self.b).dot(normal);
        let twice_cap_area = (self.a - self.c).cross(p - self.c).dot(normal);
        let twice_abp_area = (self.b - self.a).cross(p - self.a).dot(normal);

        let u = twice_bcp_area / twice_abc_area;
        let v = twice_cap_area / twice_abc_area;
        let w = twice_abp_area / twice_abc_area;

        if u < 0.0 || v < 0.0 || w < 0.0 {
            return None;
        }

        let mut rec = HitRecord {
            t,
            point: ray.at(t),
            u,
            v,
            ..Default::default()
        };
        rec.set_face_normal(ray, normal.normalize());
        rec.mat = self.mat.clone();
        Some(rec)
    }

    fn aabb(&self) -> Bounds3 {
        self.aabb
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::{self, null};

    use crate::render::{NullMaterial, null_material_ptr};

    use super::*;

    /*
    #[test]
    fn test_hit_nonintersecting_ray_returns_none() {
        let a = DVec3::ZERO;
        let b = DVec3::new(0.5, 1.0, 0.0);
        let c = DVec3::new(1.0, 0.0, 0.0);
        let tri = Triangle::new(a, b, c, null_material_ptr());
        let ray = Ray::new(DVec3::new(0.0, 0.0, 5.0), DVec3::new(0.0, 1.0, 0.0));
        assert!(tri.hit(&ray, DInterval::UNIVERSE).is_none());
    }

    #[test]
    fn test_hit_intersecting_ray_front_facing_tri_returns_valid_hit() {
        let a = DVec3::ZERO;
        let b = DVec3::new(1.0, 0.0, 0.0);
        let c = DVec3::new(0.5, 1.0, 0.0);
        let tri = Triangle::new(a, b, c, null_material_ptr());
        let ray = Ray::new(DVec3::new(0.5, 0.5, 1.0), DVec3::new(0.0, 0.0, -1.0));
        let record = tri.hit(&ray, DInterval::UNIVERSE).unwrap();
        assert_eq!(record.t, 1.0);
        assert_eq!(record.point, DVec3::new(0.5, 0.5, 0.0));
        assert_eq!(record.normal, DVec3::new(0.0, 0.0, 1.0));
        assert!(record.is_front_face);
        // TODO: should I add tests for uv?
    }

    #[test]
    fn test_hit_intersecting_ray_back_facing_tri_returns_valid_hit() {
        let a = DVec3::ZERO;
        let b = DVec3::new(1.0, 0.0, 0.0);
        let c = DVec3::new(0.5, 1.0, 0.0);
        let tri = Triangle::new(a, b, c, null_material_ptr());
        let ray = Ray::new(DVec3::new(0.5, 0.5, -1.0), DVec3::new(0.0, 0.0, 1.0));
        let record = tri.hit(&ray, DInterval::UNIVERSE).unwrap();
        assert_eq!(record.t, 1.0);
        assert_eq!(record.point, DVec3::new(0.5, 0.5, 0.0));
        assert_eq!(record.normal, DVec3::new(0.0, 0.0, -1.0));
        assert!(!record.is_front_face);
        // TODO: should I add tests for uv?
    }
    */

    #[test]
    fn test_aabb_returns_bounding_box() {
        let a = DVec3::ZERO;
        let b = DVec3::new(1.0, 0.0, 0.0);
        let c = DVec3::new(0.5, 1.0, 0.0);
        let tri = Triangle::new(a, b, c, null_material_ptr());
        assert_eq!(
            tri.aabb(),
            Bounds3::new(DVec3::new(0.0, 0.0, 0.0), DVec3::new(1.0, 1.0, 0.0))
        );
    }
}
