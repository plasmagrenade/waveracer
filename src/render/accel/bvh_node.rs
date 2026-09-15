use std::fmt;

use rand::RngExt;

use crate::{
    math::{Bounds3, DInterval, Ray},
    render::{Empty, HitRecord, Hittable, SomeHittable},
};

#[derive(Clone)]
pub struct BVHNode {
    children: Vec<SomeHittable>,
    aabb: Bounds3,
}

impl BVHNode {
    pub fn new(world: &mut [SomeHittable]) -> Self {
        let children: Vec<SomeHittable> = match world.len() {
            ..=2 => world.to_vec(),
            3.. => {
                let mid = world.len() / 2;
                let axis = rand::rng().random_range(..3);
                world.sort_by(|x, y| {
                    x.aabb().centroid()[axis]
                        .partial_cmp(&y.aabb().centroid()[axis])
                        .expect("Impossible AABB comparison")
                });
                let splits = world.split_at_mut(mid);
                vec![
                    Box::new(BVHNode::new(splits.0)),
                    Box::new(BVHNode::new(splits.1)),
                ]
            }
        };
        let aabb = children
            .iter()
            .fold(Bounds3::EMPTY, |acc, x| acc.combined(&x.aabb()));

        Self { children, aabb }
    }
}

impl Default for BVHNode {
    fn default() -> Self {
        Self::new(&mut [])
    }
}

impl Hittable for BVHNode {
    fn hit(&self, ray: &Ray, ray_t: DInterval) -> Option<HitRecord> {
        let mut closest_so_far = ray_t.max;
        let mut result: Option<HitRecord> = None;

        if !self.aabb().intersected_by(ray, ray_t) {
            return None;
        }

        for object in self.children.iter() {
            let cur_ray_t = DInterval::new(ray_t.min, closest_so_far);
            if let Some(rec) = object.hit(ray, cur_ray_t) {
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

impl fmt::Debug for BVHNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BVHNode").finish()
    }
}

#[cfg(test)]
mod tests {
    use std::default;

    use glam::DVec3;

    use crate::render::MockHittable;

    use super::*;

    /*
    #[test]
    fn test_hit_empty_node_returns_false() {
        let node = BVHNode::default();

        let ray = Ray::new(DVec3::ZERO, DVec3::new(0.0, 0.0, -1.0));
        let ray_t = DInterval::UNIVERSE;

        assert!(node.hit(&ray, ray_t).is_none());
    }

    #[test]
    fn test_hit_single_element_node_with_intersecting_ray_returns_true() {
        let mut prim = MockHittable::new();
        prim.expect_clone().returning(|| {
            let mut m = MockHittable::new();
            m.expect_aabb().returning(|| Bounds3::UNIVERSE);
            m.expect_hit()
                .returning(|_ray, _ray_t| Some(HitRecord::default()));
            m
        });

        let mut world: Vec<SomeHittable> = vec![];
        world.push(Box::new(prim));
        let node = BVHNode::new(&mut world);

        let ray = Ray::new(DVec3::ZERO, DVec3::new(0.0, 0.0, -1.0));
        let ray_t = DInterval::UNIVERSE;

        assert!(node.hit(&ray, ray_t).is_some());
    }

    #[test]
    fn test_hit_single_element_node_with_nonintersecting_ray_returns_true() {
        let mut prim = MockHittable::new();
        prim.expect_clone().returning(|| {
            let mut m = MockHittable::new();
            m.expect_aabb()
                .returning(|| Bounds3::new(DVec3::ZERO, DVec3::ZERO));
            m.expect_hit().returning(|_ray, _ray_t| None);
            m
        });

        let mut world: Vec<SomeHittable> = vec![];
        world.push(Box::new(prim));
        let node = BVHNode::new(&mut world);

        let ray = Ray::new(DVec3::ONE, DVec3::new(0.0, 0.0, -1.0));
        let ray_t = DInterval::UNIVERSE;

        assert!(node.hit(&ray, ray_t).is_none());
    }

    #[test]
    fn test_hit_two_element_node_with_one_intersecting_child_returns_true() {
        let mut prim1 = MockHittable::new();
        prim1.expect_clone().returning(|| {
            let mut m = MockHittable::new();
            m.expect_aabb().returning(|| Bounds3::UNIT);
            m.expect_hit().returning(|_ray, _ray_t| None);
            m
        });
        let mut prim2 = MockHittable::new();
        prim2.expect_clone().returning(|| {
            let mut m = MockHittable::new();
            m.expect_aabb().returning(|| Bounds3::UNIT);
            m.expect_hit()
                .returning(|_ray, _ray_t| Some(HitRecord::default()));
            m
        });

        let mut world: Vec<SomeHittable> = vec![];
        world.push(Box::new(prim1));
        world.push(Box::new(prim2));
        let node = BVHNode::new(&mut world);

        let ray = Ray::new(DVec3::new(0.0, 0.0, 1.0), DVec3::new(0.0, 0.0, -1.0));
        let ray_t = DInterval::UNIVERSE;

        assert!(node.hit(&ray, ray_t).is_some());
    }

    #[test]
    fn test_hit_two_element_node_with_two_intersecting_children_returns_hitrecord_for_closer_object()
     {
        let mut prim1 = MockHittable::new();
        prim1.expect_clone().returning(|| {
            let mut m = MockHittable::new();
            m.expect_aabb().returning(|| Bounds3::UNIT);
            m.expect_hit().returning(|_ray, _ray_t| {
                Some(HitRecord {
                    t: 2.0,
                    ..Default::default()
                })
            });
            m
        });

        let mut prim2 = MockHittable::new();
        prim2.expect_clone().returning(|| {
            let mut m = MockHittable::new();
            m.expect_aabb().returning(|| Bounds3::UNIT);
            m.expect_hit().returning(|_ray, _ray_t| {
                Some(HitRecord {
                    t: 1.0,
                    ..Default::default()
                })
            });
            m
        });

        let mut world: Vec<SomeHittable> = vec![];
        world.push(Box::new(prim1));
        world.push(Box::new(prim2));
        let node = BVHNode::new(&mut world);

        let ray = Ray::new(DVec3::new(0.0, 0.0, 1.0), DVec3::new(0.0, 0.0, -1.0));
        let ray_t = DInterval::UNIVERSE;

        assert!(node.hit(&ray, ray_t).is_some_and(|x| x.t == 1.0));
    }
    */
}
