#[repr(C)]
pub struct ColorRGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

unsafe impl cxx::ExternType for ColorRGBA {
    type Id = cxx::type_id!("ColorRGBA");
    type Kind = cxx::kind::Trivial;
}
