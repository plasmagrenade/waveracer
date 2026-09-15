use std::sync::Arc;

use glam::DVec3;

use crate::{
    math::Ray,
    render::{Material, null_material_ptr},
};

#[derive(Debug, Clone)]
pub struct HitRecord {
    pub t: f64,
    pub point: DVec3,
    pub normal: DVec3,
    // Whether we hit the outside of the object
    pub is_front_face: bool,
    // Texture UV coords
    pub u: f64,
    pub v: f64,
    pub mat: Arc<dyn Material>,
}

impl HitRecord {
    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: DVec3) {
        self.is_front_face = ray.direction().dot(outward_normal) < 0.0;
        self.normal = if self.is_front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

impl Default for HitRecord {
    fn default() -> Self {
        Self {
            t: f64::NAN,
            point: DVec3::NAN,
            normal: DVec3::NAN,
            is_front_face: false,
            u: f64::NAN,
            v: f64::NAN,
            mat: null_material_ptr(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
    #[test]
    fn test_set_face_normal_opposing_ray_sets_is_front_face_to_true() {
        let mut record = HitRecord::default();
        let normal = DVec3::new(0.0, 1.0, 0.0);
        let ray = Ray::new(DVec3::ZERO, DVec3::new(0.0, -1.0, 0.0));
        record.set_face_normal(&ray, normal);
        assert!(record.is_front_face);
        assert_eq!(record.normal, normal);
    }

    #[test]
    fn test_set_face_normal_same_direction_ray_sets_is_front_face_to_false() {
        let mut record = HitRecord::default();
        let normal = DVec3::new(0.0, 1.0, 0.0);
        let ray = Ray::new(DVec3::ZERO, DVec3::new(0.0, 1.0, 0.0));
        record.set_face_normal(&ray, normal);
        assert!(!record.is_front_face);
        assert_eq!(record.normal, -normal);
    }

    #[test]
    fn test_set_face_normal_orthogonal_ray_sets_is_front_face_to_false() {
        // Admittedly, this test is a bit contrived. We should never really have a HitRecord for a
        // "hit" where the ray is exactly orthogonal to the normal, only approaching orthogonality.
        let mut record = HitRecord::default();
        let normal = DVec3::new(0.0, 1.0, 0.0);
        let ray = Ray::new(DVec3::ZERO, DVec3::new(1.0, 0.0, 0.0));
        record.set_face_normal(&ray, normal);
        assert!(!record.is_front_face);
    }
    */
}
