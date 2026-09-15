use std::{collections::LinkedList, mem::take};

use glam::DVec3;
use kdam::{BarExt, tqdm};

use crate::math::{GpuRay, Ray};
use crate::render::{BVHNode, GpuTriangle, Triangle, null_material_ptr, run_kernel};
use crate::{
    math::{
        Color, Lerp,
        random::{random_double, random_in_unit_disk, random_in_xy_unit_square},
    },
    render::{CameraState, CameraTransition, Film, FrameData, RaySpawner, Renderer, SomeHittable},
};

pub struct VideoCamera<T> {
    pub vfov: f64,
    renderer: T,
    transitions: LinkedList<CameraTransition>,
    pub cur_frame: i32,
    pub total_frames: usize,
    pub film: Film,
    pub focus_distance: f64,
    pub defocus_angle: f64,
}

impl<T: Renderer> VideoCamera<T> {
    pub fn new(
        initial_state: &CameraState,
        vfov: f64,
        focus_distance: f64,
        defocus_angle: f64,
        renderer: T,
        film: Film,
    ) -> Self {
        // Create default single-frame hold
        let mut transitions = LinkedList::new();
        transitions.push_back(CameraTransition::new(
            Lerp::hold(initial_state.pos),
            Lerp::hold(initial_state.look_at),
            Lerp::hold(initial_state.up),
            1,
        ));

        Self {
            vfov,
            renderer,
            transitions,
            cur_frame: 0,
            total_frames: 1,
            focus_distance,
            defocus_angle,
            film,
        }
    }

    pub fn add_transition(&mut self, transition: CameraTransition) {
        self.total_frames += transition.len();
        self.transitions.push_back(transition);
    }

    pub fn render_frames(&self, world: &[SomeHittable]) -> impl Iterator<Item = FrameData> {
        self.transitions
            .clone()
            .into_iter()
            .flatten()
            .enumerate()
            .map(move |(frame, s)| self.render_frame(world, &s, frame))
    }

    pub fn render_accel_frames(&self, world: &BVHNode) -> impl Iterator<Item = FrameData> {
        self.transitions
            .clone()
            .into_iter()
            .flatten()
            .enumerate()
            .map(move |(frame, s)| self.render_frame(&[Box::new(world.clone())], &s, frame))
    }

    pub fn render_frames_gpu(&self, world: &[SomeHittable]) -> impl Iterator<Item = FrameData> {
        self.transitions
            .clone()
            .into_iter()
            .flatten()
            .enumerate()
            .map(move |(frame, s)| self.render_frame_gpu(world, &s, frame))
    }

    fn render_frame(
        &self,
        world: &[SomeHittable],
        camera_state: &CameraState,
        frame_number: usize,
    ) -> FrameData {
        let mut pixels: Vec<Color> = vec![];

        let ray_spawner = RaySpawner::new(self, camera_state);

        let mut bar = tqdm!(
            total = self.film.width * self.film.height,
            position = 1,
            desc = "  frame"
        );
        for y in 0..self.film.height {
            for x in 0..self.film.width {
                let mut pixel_color = Color::ZERO;

                // cast SAMPLES_PER_PIXEL random-ish rays and then divide total color by
                // SAMPLES_PER_PIXEL for simple antialiasing
                for _ in 0..self.film.samples_per_pixel {
                    let offset = random_in_xy_unit_square();

                    let ray = ray_spawner.generate_primary_ray(x, y, offset);

                    pixel_color += self.renderer.ray_color(&ray, world, 0);
                }
                // NOTE: I chose to not include samples in my progress bar for the sake of having a
                // semi-readable number
                bar.update(1).unwrap();
                pixels.push(pixel_color / self.film.samples_per_pixel as f64);
            }
        }

        FrameData {
            w: self.film.width,
            h: self.film.height,
            pixels,
            frame_number,
            t: 0.0,
        }
    }

    fn render_frame_gpu(
        &self,
        world: &[SomeHittable],
        camera_state: &CameraState,
        frame_number: usize,
    ) -> FrameData {
        let mut pixels: Vec<Color> = vec![];
        let mut rays: Vec<Ray> = vec![];

        let ray_spawner = RaySpawner::new(self, camera_state);

        // generate primary rays for each pixel, load into buffer
        // cast each ray into scene via gpu kernel (results in output buffer)
        // for each output result, if hit, color white, else color black
        for y in 0..self.film.height {
            for x in 0..self.film.width {
                let mut pixel_color = Color::ZERO;

                let offset = random_in_xy_unit_square();

                let ray = ray_spawner.generate_primary_ray(x, y, offset);
                // rays.push(GpuRay::from(&ray));
                rays.push(ray);
                // pixels.push(DVec3::splat(0.5));
            }
        }

        let plane_a = DVec3::new(-1.0, -0.5, -1.0);
        let plane_b = DVec3::new(-1.0, 0.5, -1.0);
        let plane_c = DVec3::new(1.0, -0.5, -1.0);
        let plane_left = Triangle::new(plane_a, plane_b, plane_c, null_material_ptr());
        let tris = &[plane_left];

        let gpu_rays: Vec<GpuRay> = rays.iter().map(GpuRay::from).collect();
        let gpu_tris: Vec<GpuTriangle> = tris.iter().map(GpuTriangle::from).collect();
        let result = run_kernel(&gpu_rays, &gpu_tris, &self.film);
        let hit_buf = result.unwrap();
        dbg!(hit_buf[0] as f64 * 255.0);
        dbg!(hit_buf[1] as f64 * 255.0);

        for hit in hit_buf {
            if hit > 0.0 {
                pixels.push(DVec3::ONE);
            } else {
                pixels.push(DVec3::ZERO);
            }
            // let pixel = DVec3::splat(hit as f64 * 255.0);
            // pixels.push(pixel);
        }
        // pixels.push(DVec3::splat(0.5));

        FrameData {
            w: self.film.width,
            h: self.film.height,
            pixels,
            frame_number,
            t: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{math::Lerp, render::MockRenderer};

    use super::*;

    #[test]
    fn test_add_transition_updates_total_frames() {
        let mut c = VideoCamera::new(
            &CameraState::default(),
            90.0,
            1.0,
            1.0,
            MockRenderer::new(),
            Film {
                width: 1,
                height: 1,
                samples_per_pixel: 1,
            },
        );
        assert_eq!(c.total_frames, 1);

        let t = CameraTransition::new(
            Lerp::hold(DVec3::ZERO),
            Lerp::hold(DVec3::ZERO),
            Lerp::hold(DVec3::ZERO),
            2,
        );
        c.add_transition(t);
        assert_eq!(c.total_frames, 3);
    }
}
