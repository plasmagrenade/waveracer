use glam::DVec3;

use crate::{
    math::{Ray, random::random_in_unit_disk},
    render::{CameraState, Renderer, VideoCamera},
};

#[derive(Debug)]
pub struct RaySpawner {
    pub pixel00_loc: DVec3,
    pub pixel_delta_u: DVec3,
    pub pixel_delta_v: DVec3,
    pub camera_center: DVec3,
    pub camera_defocus_angle: f64,
    pub camera_focus_distance: f64,
    pub defocus_disk_u: DVec3,
    pub defocus_disk_v: DVec3,
}

impl RaySpawner {
    pub fn new<T: Renderer>(camera: &VideoCamera<T>, state: &CameraState) -> Self {
        // let image_h: f64 = (self.width as f64 / (16.0 / 9.0)).max(1.0);
        let camera_center = state.pos;

        let theta = camera.vfov.to_radians();
        let h = (theta / 2.0).tan();

        let camera_focus_distance = camera.focus_distance;
        let camera_defocus_angle: f64 = camera.defocus_angle;

        let viewport_height = 2.0 * h * camera_focus_distance;
        // recalculate aspect ratio because image_h might not be what we intended
        let viewport_width =
            viewport_height * (camera.film.width as f64 / camera.film.height as f64);

        // calculate u,v,w unit basis vectors for camera
        let w = (camera_center - state.look_at).normalize();
        let u = state.up.cross(w).normalize();
        let v = w.cross(u);

        // calculate the vectors along the horizontal and vertical edges of the viewport
        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;

        // calculate the horizontal and vertical delta vectors from pixel to pixel
        let pixel_delta_u = viewport_u / camera.film.width as f64;
        let pixel_delta_v = viewport_v / camera.film.height as f64;

        // calculate the location of the upper left pixel
        let viewport_upper_left =
            camera_center - (camera_focus_distance * w) - viewport_u / 2.0 - viewport_v / 2.0;

        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        let defocus_radius =
            camera_focus_distance * (camera_defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        Self {
            camera_center,
            camera_defocus_angle,
            camera_focus_distance,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            defocus_disk_u,
            defocus_disk_v,
        }
    }

    /// Spawns a single ray at pixel [x, y] using the CameraRenderConfig's settings. For this
    /// camera the ray is offset by the defocus amount to add depth of field
    pub fn generate_primary_ray(&self, x: usize, y: usize, offset: DVec3) -> Ray {
        let pixel_center = self.pixel00_loc
            + ((x as f64 + offset.x) * self.pixel_delta_u)
            + ((y as f64 + offset.y) * self.pixel_delta_v);

        let ray_origin = if self.camera_defocus_angle <= 0.0 {
            self.camera_center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_center - ray_origin;

        Ray::new(ray_origin, ray_direction, x, y)
    }

    pub fn defocus_disk_sample(&self) -> DVec3 {
        let p = random_in_unit_disk();
        self.camera_center + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
    }
}
