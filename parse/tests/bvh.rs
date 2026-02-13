//! BVH parser tests.

#[test]
fn parse_simple() {
    let data = include_str!("fixtures/sample.bvh");
    let bvh = parse::bvh::parse_str(data).unwrap();
    assert!(!bvh.joints.is_empty());
    assert_eq!(bvh.frame_count, 2);
}
