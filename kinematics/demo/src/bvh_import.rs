//! Convert BVH motion-capture data to kinematics Armature (hierarchy from offsets).
//!
//! Enabled with the `bvh` feature. Uses fixed joints for each BVH joint so the
//! skeleton matches the BVH hierarchy and link lengths; scale factor 0.01 (cm → m).

#[cfg(feature = "bvh")]
use kinematics::{Armature, FixedJoint, JointData, JointVariant};
#[cfg(feature = "bvh")]
use mathlib::cg::vector3;

/// Scale factor from BVH units (often cm) to demo scale (m).
const BVH_SCALE: f32 = 0.01;

/// Builds a kinematics `Armature` from parsed BVH data (hierarchy only; T-pose from offsets).
///
/// Each BVH joint becomes a fixed joint with the joint's offset (scaled by `BVH_SCALE`).
/// End sites are included as zero-length links so the chain length is correct.
#[cfg(feature = "bvh")]
#[must_use]
pub fn armature_from_bvh(bvh: &parse::bvh::BvhData) -> Option<Armature> {
    let joints = &bvh.joints;
    if joints.is_empty() {
        return None;
    }
    let root_offset = &joints[0].offset;
    let root_trans = vector3(
        root_offset.get(0) * BVH_SCALE,
        root_offset.get(1) * BVH_SCALE,
        root_offset.get(2) * BVH_SCALE,
    );
    let root = JointData::new(JointVariant::Fixed(FixedJoint::new(
        root_trans,
        (0.0, 0.0, 0.0),
    )));
    let mut armature = Armature::new(root);
    for (i, bvh_joint) in joints.iter().enumerate().skip(1) {
        let parent = bvh_joint.parent as usize;
        if parent >= i {
            return None;
        }
        let off = &bvh_joint.offset;
        let trans = vector3(
            off.get(0) * BVH_SCALE,
            off.get(1) * BVH_SCALE,
            off.get(2) * BVH_SCALE,
        );
        let data = JointData::new(JointVariant::Fixed(FixedJoint::new(trans, (0.0, 0.0, 0.0))));
        armature.add_child(parent, i, data);
    }
    armature.update_kinematics();
    Some(armature)
}
