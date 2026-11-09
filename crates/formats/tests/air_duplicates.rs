use ume_formats::air::parse_air;

#[test]
fn air_first_definition_of_a_duplicated_action_wins() {
    let src = "[Begin Action 1005]\nClsn1Default: 1\n Clsn1[0] = -6,10,17,22\n1001,0, 0,0, 2\n1001,1, 0,0, 2\n\n[Begin Action 1005]\n7201,0, 0,0, 2\n7201,0, 0,0, 2\n";
    let anims = parse_air(src);
    assert_eq!(anims.len(), 1);
    let a = &anims[&1005];
    assert_eq!(a.frames[0].group, 1001);
    assert_eq!(a.frames[0].clsn1.len(), 1);
}
