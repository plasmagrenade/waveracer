use glam::DVec3;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ray {
    origin: DVec3,
    direction: DVec3,
    direction_inv: DVec3,
    pub pixel_x: usize,
    pub pixel_y: usize,
}

impl Ray {
    pub fn new(origin: DVec3, direction: DVec3, pixel_x: usize, pixel_y: usize) -> Self {
        Ray {
            origin,
            direction,
            // calculate 1.0 / direction to let us replace expensive division with cheap
            // multiplication in aabb intersection test
            direction_inv: 1.0 / direction,
            pixel_x,
            pixel_y,
        }
    }

    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + self.direction * t
    }

    pub fn origin(&self) -> DVec3 {
        self.origin
    }

    pub fn direction(&self) -> DVec3 {
        self.direction
    }

    pub fn direction_inv(&self) -> DVec3 {
        self.direction_inv
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
    #[test]
    fn test_at_t0_returns_origin() {
        let r = Ray::new(DVec3::ONE, DVec3::new(0.0, 0.0, -1.0));
        assert_eq!(r.at(0.0), DVec3::ONE);
    }

    #[test]
    fn test_at_t1_returns_origin_plus_direction() {
        let origin = DVec3::ONE;
        let dir = DVec3::new(0.0, 0.0, -1.0);
        let r = Ray::new(origin, dir);
        assert_eq!(r.at(1.0), origin + dir);
    }

    #[test]
    fn test_direction_inv_for_nonzero_dir_returns_inverse_of_dir() {
        let origin = DVec3::ONE;
        let dir = DVec3::splat(2.0);
        let r = Ray::new(origin, dir);
        assert_eq!(r.direction_inv(), DVec3::splat(0.5));
    }

    #[test]
    fn test_direction_inv_for_zero_dir_returns_infinity() {
        let origin = DVec3::ONE;
        let dir = DVec3::ZERO;
        let r = Ray::new(origin, dir);
        assert_eq!(r.direction_inv(), DVec3::splat(f64::INFINITY));
    }

    #[test]
    fn test_direction_inv_for_dir_with_some_zero_components_returns_infinity_for_only_those_components()
     {
        let origin = DVec3::ONE;
        let dir = DVec3::new(0.0, -2.0, 2.0);
        let r = Ray::new(origin, dir);
        assert_eq!(r.direction_inv(), DVec3::new(f64::INFINITY, -0.5, 0.5));
    }
    */
}
