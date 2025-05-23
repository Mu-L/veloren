use super::{
    super::{Animation, vek::*},
    QuadrupedLowSkeleton, SkeletonAttr,
};
use common::states::utils::StageSection;
//use std::ops::Rem;

pub struct ShockwaveAnimation;

impl Animation for ShockwaveAnimation {
    type Dependency<'a> = (f32, f32, Option<StageSection>, f32);
    type Skeleton = QuadrupedLowSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"quadruped_low_shockwave\0";

    #[cfg_attr(
        feature = "be-dyn-lib",
        unsafe(export_name = "quadruped_low_shockwave")
    )]
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (_velocity, global_time, stage_section, timer): Self::Dependency<'_>,
        anim_time: f32,
        _rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        let (movement1base, movement2base, movement3) = match stage_section {
            Some(StageSection::Buildup) => (anim_time.sqrt(), 0.0, 0.0),
            Some(StageSection::Action) => (1.0, anim_time.powi(4), 0.0),
            Some(StageSection::Recover) => (1.0, 1.0, anim_time),
            _ => (0.0, 0.0, 0.0),
        };
        let pullback = -1.0 - movement3;
        let subtract = global_time - timer;
        let check = subtract - subtract.trunc();
        let mirror = (check - 0.5).signum();
        let twitch3 = (mirror * movement3 * 9.0).sin();
        let movement1 = mirror * movement1base * pullback;
        let movement2 = mirror * movement2base * pullback;
        let movement1abs = movement1base * pullback;
        let movement2abs = movement2base * pullback;

        // Center head
        next.head_c_upper.orientation = Quaternion::rotation_z(twitch3 * 1.0);

        next.head_c_lower.orientation =
            Quaternion::rotation_x(movement1abs * 0.5 + movement2abs * -0.6)
                * Quaternion::rotation_y(movement1 * -0.1 + movement2 * 0.5);

        next.jaw_c.orientation = Quaternion::rotation_x(movement1abs * 0.0 + movement2abs * 0.3)
            * Quaternion::rotation_z(twitch3 * 0.2);

        // Left head
        next.head_l_upper.orientation = Quaternion::rotation_z(twitch3 * 1.0);

        next.head_l_lower.orientation =
            Quaternion::rotation_x(movement1abs * 0.5 + movement2abs * -0.6)
                * Quaternion::rotation_y(movement1 * -0.1 + movement2 * 0.5);

        next.jaw_l.orientation = Quaternion::rotation_x(movement1abs * 0.0 + movement2abs * 0.3)
            * Quaternion::rotation_z(twitch3 * 0.2);

        // Right head
        next.head_r_upper.orientation = Quaternion::rotation_z(twitch3 * 1.0);

        next.head_r_lower.orientation =
            Quaternion::rotation_x(movement1abs * 0.5 + movement2abs * -0.6)
                * Quaternion::rotation_y(movement1 * -0.1 + movement2 * 0.5);

        next.jaw_r.orientation = Quaternion::rotation_x(movement1abs * 0.0 + movement2abs * 0.3)
            * Quaternion::rotation_z(twitch3 * 0.2);

        next.chest.orientation = Quaternion::rotation_y(movement1 * 0.08 + movement2 * -0.15)
            * Quaternion::rotation_z(movement1 * 0.2 + movement2 * -0.3);
        if s_a.tongue_for_tail {
            next.tail_front.scale = Vec3::one() * 0.1;
            next.tail_rear.scale = Vec3::one() * 0.1;
        } else {
            next.tail_front.orientation = Quaternion::rotation_x(0.15)
                * Quaternion::rotation_z(movement1 * 0.2 + movement2 * 0.2)
                * twitch3
                * 0.8;

            next.tail_rear.orientation = Quaternion::rotation_x(-0.12)
                * Quaternion::rotation_z(movement1 * 0.2 + movement2 * 0.2)
                * twitch3
                * 0.8;
        }
        next
    }
}
