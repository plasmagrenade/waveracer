#![allow(dead_code)]

use std::fmt;
use std::sync::Arc;

use glam::{DVec3, IVec3};

use crate::{
    math::{ALMOST_ZERO, Bounds3, DInterval, Ray},
    render::{Hittable, Material, SomeHittable, Triangle},
};

use super::HitRecord;

#[derive(Clone)]
pub struct TriangleMesh {
    vertices: Vec<DVec3>,
    triangles: Vec<IVec3>,
    // TODO: probably refactor this so that it can be accelerated on a per-triangle basis
    cache: Vec<SomeHittable>,
    aabb: Bounds3,
}

impl fmt::Debug for TriangleMesh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TriangleMesh")
            .field("vertices", &self.vertices)
            .field("triangles", &self.triangles)
            .finish()
    }
}

impl TriangleMesh {
    pub fn new(vertices: Vec<DVec3>, triangles: Vec<IVec3>, mat: Arc<dyn Material>) -> Self {
        let mut cache: Vec<SomeHittable> = vec![];
        for triangle in &triangles {
            let a = vertices[triangle.x as usize];
            let b = vertices[triangle.y as usize];
            let c = vertices[triangle.z as usize];
            // will this take up unnecessary space? could it be denormalized?
            // TODO: revisit this if I get *really* nitpicky about performance
            let tri = Triangle::new(a, b, c, mat.clone());
            cache.push(Box::new(tri));
        }
        let aabb = Bounds3::new(
            cache
                .iter()
                .fold(DVec3::MAX, |cur_min, tri| cur_min.min(tri.aabb().min)),
            cache
                .iter()
                .fold(DVec3::MIN, |cur_max, tri| cur_max.max(tri.aabb().max)),
        );

        Self {
            vertices,
            triangles,
            cache,
            aabb,
        }
    }
}

impl Hittable for TriangleMesh {
    fn hit(&self, ray: &Ray, ray_t: DInterval) -> Option<HitRecord> {
        if !self.aabb().intersected_by(ray, ray_t) {
            return None;
        }

        let mut closest_so_far = ray_t.max;
        let mut result: Option<HitRecord> = None;

        for triangle in &self.cache {
            if let Some(rec) = triangle.hit(ray, DInterval::new(ray_t.min, closest_so_far)) {
                closest_so_far = rec.t;
                result = Some(rec);
            }
        }
        result
    }

    fn aabb(&self) -> Bounds3 {
        self.aabb
    }
}

#[cfg(test)]
mod tests {

    use crate::render::null_material_ptr;

    use super::*;

    /*
    #[test]
    fn test_hit_nonintersecting_ray_returns_none() {
        let a = DVec3::new(-1.0, -1.0, 1.0);
        let b = DVec3::new(-1.0, 1.0, 1.0);
        let c = DVec3::new(1.0, 1.0, 1.0);
        let d = DVec3::new(1.0, -1.0, 1.0);

        let mesh = TriangleMesh::new(
            vec![a, b, c, d],
            vec![IVec3::new(0, 1, 2), IVec3::new(2, 3, 0)],
            null_material_ptr(),
        );

        let ray = Ray::new(DVec3::ZERO, DVec3::new(0.0, 0.0, -1.0));

        assert!(mesh.hit(&ray, DInterval::new(0.0, 10.0)).is_none());
    }

    #[test]
    fn test_hit_intersecting_ray_front_facing_mesh_returns_valid_hit() {
        let a = DVec3::new(-1.0, -1.0, 1.0);
        let b = DVec3::new(-1.0, 1.0, 1.0);
        let c = DVec3::new(1.0, 1.0, 1.0);
        let d = DVec3::new(1.0, -1.0, 1.0);

        let mesh = TriangleMesh::new(
            vec![a, b, c, d],
            vec![IVec3::new(0, 1, 2), IVec3::new(2, 3, 0)],
            null_material_ptr(),
        );

        let ray = Ray::new(DVec3::ZERO, DVec3::new(0.0, 0.0, 1.0));
        let hit = mesh.hit(&ray, DInterval::UNIVERSE).unwrap();

        assert_eq!(hit.t, 1.0);
        assert_eq!(hit.point, DVec3::new(0.0, 0.0, 1.0));
        assert_eq!(hit.normal, DVec3::new(0.0, 0.0, -1.0));
        assert!(hit.is_front_face);
    }

    #[test]
    fn test_hit_intersecting_ray_back_facing_mesh_returns_valid_hit() {
        let a = DVec3::new(-1.0, -1.0, 1.0);
        let b = DVec3::new(-1.0, 1.0, 1.0);
        let c = DVec3::new(1.0, 1.0, 1.0);
        let d = DVec3::new(1.0, -1.0, 1.0);

        let mesh = TriangleMesh::new(
            vec![a, b, c, d],
            vec![IVec3::new(0, 1, 2), IVec3::new(2, 3, 0)],
            null_material_ptr(),
        );

        let ray = Ray::new(DVec3::new(0.0, 0.0, 2.0), DVec3::new(0.0, 0.0, -1.0));
        let hit = mesh.hit(&ray, DInterval::UNIVERSE).unwrap();

        assert_eq!(hit.t, 1.0);
        assert_eq!(hit.point, DVec3::new(0.0, 0.0, 1.0));
        assert_eq!(hit.normal, DVec3::new(0.0, 0.0, 1.0));
        assert!(!hit.is_front_face);
    }

    #[test]
    fn test_aabb() {
        let a = DVec3::new(-1.0, -1.0, 1.0);
        let b = DVec3::new(-1.0, 1.0, 1.0);
        let c = DVec3::new(1.0, 1.0, 1.0);
        let d = DVec3::new(1.0, -1.0, 1.0);

        let mesh = TriangleMesh::new(
            vec![a, b, c, d],
            vec![IVec3::new(0, 1, 2), IVec3::new(2, 3, 0)],
            null_material_ptr(),
        );

        let expected_min = DVec3::new(-1.0, -1.0, 1.0);
        let expected_max = DVec3::new(1.0, 1.0, 1.0);
        assert_eq!(mesh.aabb(), Bounds3::new(expected_min, expected_max));
    }
    */
}
