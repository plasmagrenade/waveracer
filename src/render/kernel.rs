use ezgpu::{Buffer, Gpu};

use crate::{
    math::GpuRay,
    render::{Film, GpuTriangle},
};

pub fn run_kernel(rays: &[GpuRay], tris: &[GpuTriangle], film: &Film) -> ezgpu::Result<Vec<f32>> {
    dbg!(rays[1]);
    let gpu = Gpu::new()?;

    let kernel = gpu.kernel(
        r#"
        struct Ray { origin: vec3<f32>, direction: vec3<f32>, direction_inv: vec3<f32>, pixel_x: u32, pixel_y: u32 }
        struct Triangle { a: vec3<f32>, b: vec3<f32>, c: vec3<f32> }

        @group(0) @binding(0) var<storage, read>       rays:    array<Ray>;
        @group(0) @binding(1) var<storage, read>       tris:    array<Triangle>;
        @group(0) @binding(2) var<storage, read_write> hits:    array<f32>;
        @group(0) @binding(3) var<uniform>             width:   u32;
        @group(0) @binding(4) var<uniform>             height:  u32;

        @compute @workgroup_size(8, 8)
        fn main(@builtin(global_invocation_id) id: vec3<u32>) {
            if id.x >= width || id.y >= height { return; }
            let index = id.y * width + id.x;

            // if index >= arrayLength(&rays) { return; }
            let ray = rays[index];

            var nearest = -1.0;
            for (var t_index = 0u; t_index < arrayLength(&tris); t_index++) {
                let tri = tris[t_index];
                let ab = tri.b - tri.a;
                let ac = tri.c - tri.a;

                // NOTE: we don't normalize this because we're using the length squared as a shortcut for
                // "area of the parallelogram defined by AB and AC"
                let normal = cross(ab, ac);

                // NOTE: this is reversed from plane.rs, since ray.direction is unit length but normal isn't
                let denom = dot(normal, ray.direction);

                // ray is parallel to the triangle - ray projected onto normal approaches zero
                if abs(denom) < 0.000001 {
                    continue;
                }

                // find ray intersection with plane
                let t = dot(normal, tri.a - ray.origin) / denom;

                if !(t > 0.0 && (nearest < 0.0 || t < nearest)) {
                    continue;
                }

                let p = ray.origin + ray.direction * t;

                let twice_abc_area = dot(normal, normal);
                let twice_bcp_area = dot(cross(tri.c - tri.b, p - tri.b), normal);
                let twice_cap_area = dot(cross(tri.a - tri.c, p - tri.c), normal);
                let twice_abp_area = dot(cross(tri.b - tri.a, p - tri.a), normal);

                let u = twice_bcp_area / twice_abc_area;
                let v = twice_cap_area / twice_abc_area;
                let w = twice_abp_area / twice_abc_area;

                if u < 0.0 || v < 0.0 || w < 0.0 {
                    continue;
                }

                nearest = t;
            }
            hits[index] = nearest;
        }
        "#,
    )?;

    let ray_buf = gpu.buffer(&rays);
    let tri_buf = gpu.buffer(&tris);
    let hit_buf = gpu.empty::<f32>(rays.len());

    let needed = (rays.len() as f64 / 64.0).ceil() as u64;
    let workgroups_x = needed.min(65535);
    let workgroups_y = (needed as f64 / workgroups_x as f64).ceil() as u64;
    assert!(workgroups_x <= 65535 && workgroups_y <= 65535);

    kernel
        .bind("rays", &ray_buf)
        .bind("tris", &tri_buf)
        .bind("hits", &hit_buf)
        .bind("width", &gpu.uniform(film.width))
        .bind("height", &gpu.uniform(film.height))
        .dispatch_elements((film.width, film.height))?;

    let foo = hit_buf.read();

    foo
}
