use std::sync::Arc;

use glam::{DVec3, IVec3};

use crate::{
    math::Color,
    render::{
        CameraState, DiffuseLight, Film, Renderer, SolidColor, SomeHittable, Triangle, VideoCamera,
    },
    scene::SceneData,
};

pub fn idiot<T: Renderer>(renderer: T, film: Film) -> SceneData<T> {
    let start = CameraState {
        pos: DVec3::new(-1.711963, 4.333467, 10.445557),
        look_at: DVec3::new(0.421718, 0.592129, -2.875471),
        up: DVec3::new(0.042266, 0.96363, -0.263875),
    };
    let camera = VideoCamera::new(&start, 73.7398, 1.0, 0.0, renderer, film);

    let mut world: Vec<SomeHittable> = vec![];

    let light_tex_1 = Arc::new(SolidColor::new(Color::splat(10.0)));
    let light_mat_1 = Arc::new(DiffuseLight::new(light_tex_1));

    let plane_a = DVec3::new(-1.0, -0.5, -1.0);
    let plane_b = DVec3::new(-1.0, 0.5, -1.0);
    let plane_c = DVec3::new(1.0, -0.5, -1.0);
    let plane_left = Triangle::new(plane_a, plane_b, plane_c, light_mat_1);

    world.push(Box::new(plane_left));

    SceneData {
        world,
        name: String::from("idiot"),
        camera,
    }
}
