use crate::math::Ray;

// The GPU twin: f32, `#[repr(C)]`, explicit vec3 padding, `Pod`.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuRay {
    origin: [f32; 3],
    _pad0: f32,
    direction: [f32; 3],
    _pad1: f32,
    direction_inv: [f32; 3],
    _pad2: f32,
    pixel_x: u32,
    pixel_y: u32,
    _pad3: [u32; 2],
}

const _: () = assert!(std::mem::size_of::<GpuRay>() == 64);

impl From<&Ray> for GpuRay {
    fn from(r: &Ray) -> Self {
        GpuRay {
            origin: r.origin().as_vec3().to_array(),
            direction: r.direction().as_vec3().to_array(),
            direction_inv: r.direction_inv().as_vec3().to_array(),
            pixel_x: r.pixel_x as u32,
            pixel_y: r.pixel_y as u32,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            _pad3: [0, 0],
        }
    }
}
