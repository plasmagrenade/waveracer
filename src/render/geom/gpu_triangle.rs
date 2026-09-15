use crate::render::Triangle;

// The GPU twin: f32, `#[repr(C)]`, explicit vec3 padding, `Pod`.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuTriangle {
    a: [f32; 3],
    _pad0: f32,
    b: [f32; 3],
    _pad1: f32,
    c: [f32; 3],
    _pad2: f32,
}

const _: () = assert!(std::mem::size_of::<GpuTriangle>() == 48);

impl From<&Triangle> for GpuTriangle {
    fn from(t: &Triangle) -> Self {
        GpuTriangle {
            a: t.a.as_vec3().to_array(),
            b: t.b.as_vec3().to_array(),
            c: t.c.as_vec3().to_array(),
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        }
    }
}
