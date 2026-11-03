#[test]
fn sprite_shader_validates() {
    let module = naga::front::wgsl::parse_str(ume_render::SHADER).expect("wgsl parse");
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::empty()).validate(&module).expect("wgsl validate");
}
