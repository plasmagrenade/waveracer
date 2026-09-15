#![allow(dead_code)]

use glam::{BVec3, DVec3};

use crate::math::{DInterval, Ray};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Bounds3 {
    pub min: DVec3,
    pub max: DVec3,
    centroid: DVec3,
}

impl Default for Bounds3 {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Bounds3 {
    pub fn new(min: DVec3, max: DVec3) -> Self {
        if !min.is_finite() {
            panic!("can't create an infinite AABB. use UNIVERSE instead");
        }
        if !max.is_finite() {
            panic!("can't create an infinite AABB. use UNIVERSE instead");
        }
        if min.cmpgt(max).any() {
            panic!("can't create a negative sized AABB. use EMPTY instead");
        }
        let centroid = min + (max - min) / 2.0;
        Self { min, max, centroid }
    }

    pub fn combined(&self, other: &Bounds3) -> Self {
        Bounds3::new(self.min.min(other.min), self.max.max(other.max))
    }

    pub fn combine_with(&mut self, other: &Bounds3) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self.centroid = self.min + (self.max - self.min) / 2.0;
    }

    pub const EMPTY: Bounds3 = Bounds3 {
        min: DVec3::MAX,
        max: DVec3::MIN,
        centroid: DVec3::ZERO,
    };

    pub const UNIVERSE: Bounds3 = Bounds3 {
        min: DVec3::MIN,
        max: DVec3::MAX,
        centroid: DVec3::ZERO,
    };

    pub const UNIT: Bounds3 = Bounds3 {
        min: DVec3::NEG_ONE,
        max: DVec3::ONE,
        centroid: DVec3::ZERO,
    };

    pub fn contains(&self, pt: &DVec3) -> bool {
        pt.cmpge(self.min).all() && pt.cmple(self.max).all()
    }

    pub fn intersected_by(&self, ray: &Ray, ray_t: DInterval) -> bool {
        let mut t_min = 0_f64;
        let mut t_max = f64::MAX;

        for i in 0..3 {
            let t1 = (self.min[i] - ray.origin()[i]) * ray.direction_inv()[i];
            let t2 = (self.max[i] - ray.origin()[i]) * ray.direction_inv()[i];
            // NOTE: I'm doing this instead of the more common `t_min = t_min.max(t1.min(t2))`
            // because ordering it this way fixes "exactly along border" intersections - see
            // https://tavianator.com/2022/ray_box_boundary.html#boundaries
            t_min = t1.max(t_min).min(t2.max(t_min));
            t_max = t1.min(t_max).max(t2.min(t_max));
        }

        let t_min_valid = ray_t.contains(t_min);
        let t_max_valid = ray_t.contains(t_max);
        // Special case: if ray_t is entirely inside of t_min and t_max, the ray still intersects
        // the AABB. If a ray is fired off inside of a box, it still intersects that box even if it
        // doesn't reach either edge.
        let ray_t_inside_bounds = ray_t.min >= t_min && ray_t.max <= t_max;
        t_max >= t_min && (t_min_valid || t_max_valid || ray_t_inside_bounds)
    }

    /// Get the position of a point pt relative to a bounding box, where a point at exactly
    /// bounds.min has offset (0, 0, 0), a point at exactly bounds.max has offset (1, 1, 1), and a
    /// point exactly halfway between min and max has offset (0.5, 0.5, 0.5).
    ///
    /// A point outside of the bounding box will give a scaled multiple - a point at min + 2*max
    /// will have offset (2, 2, 2), and one at min - max will have offset (-1, -1, -1).
    pub fn offset(&self, pt: &DVec3) -> DVec3 {
        let o = pt - self.min;
        let size = self.diagonal();
        // If any dim is zero-width, don't divide by that dim
        let safe = size.cmpgt(DVec3::ZERO);
        // This is kind of arbitrary. In the case that an AABB has a zero-width dimension, any
        // centroid in that dimension will have the same offset. Intuitively, this feels like it
        // should be 0.5, so it is.
        DVec3::select(safe, o / size, DVec3::splat(0.5))
    }

    pub fn centroid(&self) -> DVec3 {
        self.centroid
    }

    pub fn diagonal(&self) -> DVec3 {
        self.max - self.min
    }

    pub fn surface_area(&self) -> f64 {
        let d = self.diagonal();

        2.0 * (d.x * d.y + d.x * d.z + d.y * d.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic = "can't create an infinite AABB. use UNIVERSE instead"]
    fn test_new_panics_with_infinite_min() {
        Bounds3::new(DVec3::new(0.0, 0.0, f64::NEG_INFINITY), DVec3::ZERO);
    }

    #[test]
    #[should_panic = "can't create an infinite AABB. use UNIVERSE instead"]
    fn test_new_panics_with_infinite_max() {
        Bounds3::new(DVec3::ZERO, DVec3::new(f64::INFINITY, 0.0, 0.0));
    }

    #[test]
    #[should_panic = "can't create a negative sized AABB"]
    fn test_new_panics_with_negative_size() {
        Bounds3::new(DVec3::ZERO, DVec3::new(1.0, -1.0, 0.0));
    }

    #[test]
    fn test_combined_uses_smaller_min_and_larger_max() {
        let first = Bounds3::UNIT;
        let second = Bounds3::new(DVec3::new(-0.9, -2.0, -1.0), DVec3::new(0.0, 1.0, 1.1));

        let result = Bounds3::combined(&first, &second);
        assert_eq!(
            result.min,
            DVec3::new(first.min.x, second.min.y, first.min.z)
        );
        assert_eq!(
            result.max,
            DVec3::new(first.max.x, first.max.y, second.max.z)
        );
    }

    #[test]
    fn test_combined_with_completely_enveloped_bounds_uses_outer_bounds() {
        let first = Bounds3::UNIT;
        let second = Bounds3::new(DVec3::new(-0.1, -0.1, -0.1), DVec3::new(0.1, 0.1, 0.1));

        let result = Bounds3::combined(&first, &second);
        assert_eq!(result.min, first.min);
        assert_eq!(result.max, first.max);
    }

    #[test]
    fn test_combined_with_empty_and_universe_returns_universe() {
        let result = Bounds3::combined(&Bounds3::EMPTY, &Bounds3::UNIVERSE);
        assert_eq!(result.min, Bounds3::UNIVERSE.min);
        assert_eq!(result.max, Bounds3::UNIVERSE.max);
    }

    #[test]
    #[should_panic = "can't create a negative sized AABB"]
    fn test_combined_with_empty_and_empty_panics() {
        Bounds3::combined(&Bounds3::EMPTY, &Bounds3::EMPTY);
    }

    #[test]
    fn test_combine_with_recalculates_centroid() {
        let mut first = Bounds3::UNIT;
        assert_eq!(first.centroid, DVec3::ZERO);

        let second = Bounds3::new(DVec3::ONE, DVec3::splat(2.0));
        assert_eq!(second.centroid, DVec3::splat(1.5));

        first.combine_with(&second);
        assert_eq!(first.centroid, DVec3::splat(0.5));
    }

    #[test]
    fn test_contains_point_inside_returns_true() {
        assert!(Bounds3::UNIT.contains(&DVec3::ZERO));
    }

    #[test]
    fn test_contains_point_outside_returns_false() {
        let pt = DVec3::new(1.1, 0.0, 0.0);

        assert!(!Bounds3::UNIT.contains(&pt));
    }

    #[test]
    fn test_contains_point_exactly_on_min_border_returns_true() {
        assert!(Bounds3::UNIT.contains(&DVec3::NEG_ONE));
    }

    #[test]
    fn test_contains_point_exactly_on_max_border_returns_true() {
        assert!(Bounds3::UNIT.contains(&DVec3::ONE));
    }

    /*
    #[test]
    fn test_intersected_by_intersecting_ray_returns_true() {
        let ray_origin = DVec3::ZERO;
        let ray_dir = DVec3::new(0.0, 0.0, 1.0);
        let ray_t = DInterval::UNIVERSE;
        let ray = Ray::new(ray_origin, ray_dir);

        let bounds_min = DVec3::new(-1.0, -1.0, 1.0);
        let bounds_max = DVec3::new(1.0, 1.0, 3.0);
        let bounds = Bounds3::new(bounds_min, bounds_max);

        assert!(bounds.intersected_by(&ray, ray_t));
    }

    #[test]
    fn test_intersected_by_non_intersecting_ray_returns_false() {
        let ray_origin = DVec3::new(-1.5, 0.0, 0.0);
        let ray_dir = DVec3::new(0.0, 0.0, 1.0);
        let ray_t = DInterval::UNIVERSE;
        let ray = Ray::new(ray_origin, ray_dir);

        let bounds_min = DVec3::new(-1.0, -1.0, 1.0);
        let bounds_max = DVec3::new(1.0, 1.0, 3.0);
        let bounds = Bounds3::new(bounds_min, bounds_max);

        assert!(!bounds.intersected_by(&ray, ray_t));
    }

    #[test]
    fn test_intersected_by_ray_exactly_on_border_returns_true() {
        let ray_origin = DVec3::new(-1.0, 0.0, 0.0);
        let ray_dir = DVec3::new(0.0, 0.0, 1.0);
        let ray_t = DInterval::UNIVERSE;
        let ray = Ray::new(ray_origin, ray_dir);

        let bounds_min = DVec3::new(-1.0, -1.0, 1.0);
        let bounds_max = DVec3::new(1.0, 1.0, 3.0);
        let bounds = Bounds3::new(bounds_min, bounds_max);

        assert!(bounds.intersected_by(&ray, ray_t));
    }

    #[test]
    fn test_intersected_by_ray_exactly_on_corner_returns_true() {
        let ray_origin = DVec3::new(0.0, 2.0, 0.0);
        let ray_dir = DVec3::new(1.0, -1.0, -1.0);
        let ray_t = DInterval::new(0.0, 10.0);
        let ray = Ray::new(ray_origin, ray_dir);

        let bounds_min = DVec3::NEG_ONE;
        let bounds_max = DVec3::ONE;
        let bounds = Bounds3::new(bounds_min, bounds_max);

        assert!(bounds.intersected_by(&ray, ray_t));
    }

    #[test]
    fn test_intersected_by_ray_outside_of_ray_t_returns_false() {
        let ray_origin = DVec3::ZERO;
        let ray_dir = DVec3::new(0.0, 0.0, 1.0);
        let ray_t = DInterval::new(100.0, 1000.0);
        let ray = Ray::new(ray_origin, ray_dir);

        let bounds_min = DVec3::new(-1.0, -1.0, 1.0);
        let bounds_max = DVec3::new(1.0, 1.0, 3.0);
        let bounds = Bounds3::new(bounds_min, bounds_max);

        assert!(!bounds.intersected_by(&ray, ray_t));
    }

    #[test]
    fn test_intersected_by_ray_with_one_intersection_outside_of_ray_t_returns_true() {
        let ray_origin = DVec3::ZERO;
        let ray_dir = DVec3::new(0.0, 0.0, 1.0);
        let ray_t = DInterval::new(0.0, 2.0);
        let ray = Ray::new(ray_origin, ray_dir);

        let bounds_min = DVec3::new(-1.0, -1.0, -1.0);
        let bounds_max = DVec3::ONE;
        let bounds = Bounds3::new(bounds_min, bounds_max);

        assert!(bounds.intersected_by(&ray, ray_t));
    }

    #[test]
    fn test_intersected_by_ray_with_ray_t_completely_inside_bounds_returns_true() {
        let ray_origin = DVec3::ZERO;
        let ray_dir = DVec3::new(0.0, 0.0, 1.0);
        let ray_t = DInterval::new(1.1, 1.5);
        let ray = Ray::new(ray_origin, ray_dir);

        let bounds_min = DVec3::new(-1.0, -1.0, 1.0);
        let bounds_max = DVec3::new(1.0, 1.0, 2.0);
        let bounds = Bounds3::new(bounds_min, bounds_max);

        assert!(bounds.intersected_by(&ray, ray_t));
    }
    */

    #[test]
    fn test_offset_pt_at_min_returns_zeroes() {
        let bounds = Bounds3::UNIT;

        let pt = bounds.min;

        assert_eq!(bounds.offset(&pt), DVec3::ZERO);
    }

    #[test]
    fn test_offset_pt_at_max_returns_ones() {
        let bounds = Bounds3::UNIT;

        let pt = bounds.max;

        assert_eq!(bounds.offset(&pt), DVec3::ONE);
    }

    #[test]
    fn test_offset_pt_at_center_returns_point_fives() {
        let bounds = Bounds3::UNIT;

        let pt = bounds.centroid();

        assert_eq!(bounds.offset(&pt), DVec3::splat(0.5));
    }

    #[test]
    fn test_offset_pt_not_directly_in_between_min_and_max_returns_correct_value() {
        let bounds = Bounds3::UNIT;

        let pt = DVec3::new(-1.0, 1.0, 3.0);

        assert_eq!(bounds.offset(&pt), DVec3::new(0.0, 1.0, 2.0));
    }

    #[test]
    fn test_offset_pt_before_min_returns_scaled_value() {
        let bounds = Bounds3::UNIT;

        let pt = DVec3::splat(-3.0);

        assert_eq!(bounds.offset(&pt), DVec3::NEG_ONE);
    }

    #[test]
    fn test_offset_pt_past_max_returns_scaled_value() {
        let bounds = Bounds3::UNIT;

        let pt = DVec3::splat(3.0);

        assert_eq!(bounds.offset(&pt), DVec3::splat(2.0));
    }

    #[test]
    fn test_offset_with_zero_dimension_returns_point_five() {
        let bounds = Bounds3::new(DVec3::new(0.0, -1.0, -1.0), DVec3::new(0.0, 1.0, 1.0));
        let pt = DVec3::splat(3.0);
        assert_eq!(bounds.offset(&pt), DVec3::new(0.5, 2.0, 2.0));
    }

    #[test]
    fn test_centroid_returns_correct_value() {
        let bounds = Bounds3::new(DVec3::NEG_ONE, DVec3::ONE);
        assert_eq!(bounds.centroid(), DVec3::ZERO);

        let bounds2 = Bounds3::new(DVec3::ZERO, DVec3::ONE);
        assert_eq!(bounds2.centroid(), DVec3::splat(0.5));
    }

    #[test]
    fn test_centroid_universe_returns_origin() {
        let bounds = Bounds3::UNIVERSE;
        assert_eq!(bounds.centroid(), DVec3::ZERO);
    }

    #[test]
    fn test_diagonal_returns_correct_vector() {
        assert_eq!(Bounds3::UNIT.diagonal(), DVec3::splat(2.0));
        let offset_bounds = Bounds3::new(DVec3::ZERO, DVec3::ONE);
        assert_eq!(offset_bounds.diagonal(), DVec3::splat(1.0));
    }

    #[test]
    fn test_surface_area_returns_correct_surface_area() {
        assert_eq!(Bounds3::UNIT.surface_area(), 24.0);
        let rectangular_bounds = Bounds3::new(DVec3::ZERO, DVec3::new(1.0, 2.0, 3.0));
        assert_eq!(rectangular_bounds.surface_area(), 22.0);
    }
}
