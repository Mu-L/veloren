use super::{
    super::{Animation, vek::*},
    BipedSmallSkeleton, SkeletonAttr,
};
use common::{comp::item::ToolKind, states::utils::StageSection};
use std::f32::consts::PI;

pub struct ShockwaveAnimation;

type ShockwaveAnimationDependency = (
    Option<ToolKind>,
    Option<ToolKind>,
    Vec3<f32>,
    Vec3<f32>,
    Vec3<f32>,
    f32,
    Vec3<f32>,
    f32,
    Option<StageSection>,
    f32,
);

impl Animation for ShockwaveAnimation {
    type Dependency<'a> = ShockwaveAnimationDependency;
    type Skeleton = BipedSmallSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"biped_small_shockwave\0";

    #[cfg_attr(feature = "be-dyn-lib", unsafe(export_name = "biped_small_shockwave"))]

    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (
            active_tool_kind,
            second_tool_kind,
            _velocity,
            _orientation,
            _last_ori,
            global_time,
            _avg_vel,
            _acc_vel,
            stage_section,
            timer,
        ): Self::Dependency<'_>,
        anim_time: f32,
        _rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        let anim_time = anim_time.min(1.0);
        let (move1base, twitch, move2base, move3) = match stage_section {
            Some(StageSection::Buildup) => (anim_time.sqrt(), (anim_time * 13.0).sin(), 0.0, 0.0),
            Some(StageSection::Action) => (1.0, 1.0, anim_time.powi(4), 0.0),
            Some(StageSection::Recover) => (1.0, 1.0, 1.0, anim_time),
            _ => (0.0, 0.0, 0.0, 0.0),
        };
        let pullback = 1.0 - move3;
        let twitch = twitch * pullback;
        let subtract = global_time - timer;
        let check = subtract - subtract.trunc();
        let mirror = (check - 0.5).signum();
        let move1 = move1base * pullback * mirror;
        let move2 = move2base * pullback * mirror;
        let move1abs = move1base * pullback;
        let move2abs = move2base * pullback;
        next.hand_l.position = Vec3::new(s_a.grip.0 * 4.0, 0.0, s_a.grip.2);
        next.hand_r.position = Vec3::new(-s_a.grip.0 * 4.0, 0.0, s_a.grip.2);
        next.main.position = Vec3::new(0.0, 0.0, 0.0);
        next.main.orientation = Quaternion::rotation_x(0.0);
        next.second.position = Vec3::new(0.0, 0.0, 0.0);
        next.second.orientation = Quaternion::rotation_x(0.0);
        next.hand_l.orientation = Quaternion::rotation_x(0.0);
        next.hand_r.orientation = Quaternion::rotation_x(0.0);
        match (active_tool_kind, second_tool_kind) {
            (Some(ToolKind::Staff), None) => {
                next.control_l.position = Vec3::new(2.0 - s_a.grip.0 * 2.0, 1.0, 3.0);
                next.control_r.position = Vec3::new(
                    7.0 + s_a.grip.0 * 2.0 + move1abs * -8.0,
                    -4.0 + move1abs * 5.0,
                    3.0,
                );

                next.control.position = Vec3::new(
                    -5.0 + move1abs * 5.0,
                    -1.0 + s_a.grip.2 + move1abs * 3.0 + move2abs * 5.0,
                    -2.0 + -s_a.grip.2 / 2.5
                        + s_a.grip.0 * -2.0
                        + move1abs * 12.0
                        + move2abs * -13.0,
                );

                next.control_l.orientation = Quaternion::rotation_x(PI / 2.0)
                    * Quaternion::rotation_y(-0.3)
                    * Quaternion::rotation_z(-0.3);
                next.control_r.orientation = Quaternion::rotation_x(PI / 2.0 + s_a.grip.0 * 0.2)
                    * Quaternion::rotation_y(-0.4 + s_a.grip.0 * 0.2)
                    * Quaternion::rotation_z(-0.0);

                next.control.orientation = Quaternion::rotation_x(-0.3 + move1abs * 0.2)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(0.5);
                next.chest.orientation = Quaternion::rotation_x(move1abs * 0.5 + move2abs * -1.0);
                next.head.orientation =
                    Quaternion::rotation_x(move1abs * 0.5) * Quaternion::rotation_y(twitch * 0.5);
                next.main.orientation = Quaternion::rotation_z(move1abs * 0.5 + move2abs * 0.9);
            },
            (Some(ToolKind::Sword), None) => {
                let rotate = move1 * 2.0 * PI + move2 * 2.0 * PI;
                let jump = move1 + 3.0 + move2 * -3.0;
                next.control_l.position = Vec3::new(2.0 - s_a.grip.0 * 2.0, 1.0, 3.0);
                next.control_r.position = Vec3::new(
                    7.0 + s_a.grip.0 * 2.0 + move1abs * -8.0,
                    -4.0 + move1abs * 5.0,
                    3.0,
                );

                next.control.position = Vec3::new(
                    -5.0 + move1abs * 5.0,
                    -1.0 + s_a.grip.2 + move1abs * 3.0 + move2abs * 5.0,
                    -2.0 + -s_a.grip.2 / 2.5
                        + s_a.grip.0 * -2.0
                        + move1abs * 12.0
                        + move2abs * -13.0,
                );

                next.control_l.orientation = Quaternion::rotation_x(PI / 2.0)
                    * Quaternion::rotation_y(-0.3)
                    * Quaternion::rotation_z(-0.3);
                next.control_r.orientation = Quaternion::rotation_x(PI / 2.0 + s_a.grip.0 * 0.2)
                    * Quaternion::rotation_y(-0.4 + s_a.grip.0 * 0.2)
                    * Quaternion::rotation_z(-0.0);

                next.control.orientation = Quaternion::rotation_x(-0.3 + move1abs * 0.2)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(0.5);
                next.chest.orientation = Quaternion::rotation_x(move1abs * 0.5 + move2abs * -1.0);
                next.head.orientation =
                    Quaternion::rotation_x(move1abs * 0.5) * Quaternion::rotation_y(twitch * 0.5);
                next.main.orientation = Quaternion::rotation_z(move1abs * 0.5 + move2abs * 0.9);
                next.chest.orientation = Quaternion::rotation_z(rotate + (1.0 - move3));

                next.chest.position = Vec3::new(0.0, s_a.chest.0, s_a.chest.1 + jump);
            },
            (Some(ToolKind::Axe), Some(ToolKind::Axe)) => {
                next.main.position =
                    Vec3::new(-s_a.hand.0 - 4.0, s_a.hand.1 + 2.0, s_a.hand.2 - 4.0);
                next.main.orientation = Quaternion::rotation_x(-0.2);
                next.hand_l.position = Vec3::new(-s_a.hand.0, s_a.hand.1, s_a.hand.2);
                next.hand_l.orientation = Quaternion::rotation_x(1.2);
                next.second.position =
                    Vec3::new(s_a.hand.0 + 4.0, s_a.hand.1 + 2.0, s_a.hand.2 - 4.0);
                next.second.orientation = Quaternion::rotation_x(-0.2);
                next.hand_r.position = Vec3::new(s_a.hand.0, s_a.hand.1, s_a.hand.2);
                next.hand_r.orientation = Quaternion::rotation_x(1.2);

                next.chest.orientation = Quaternion::rotation_x(move2abs * -1.0)
                    * Quaternion::rotation_z(move1 * 1.2 + move2 * -1.8);

                if mirror > 0.0 {
                    next.control_r.position += Vec3::new(-5.0, 0.0, 5.0) * move1abs;
                    next.control_r.orientation.rotate_x(PI / 10.0 * move1abs);
                    next.control_l.position += Vec3::new(3.0, 0.0, 10.0) * move1abs;
                    next.main.position += Vec3::new(3.0, 0.0, 10.0) * move1abs;
                    next.control_l.orientation.rotate_x(PI / 4.0 * move1abs);
                    next.main.orientation.rotate_x(PI / 4.0 * move1abs);

                    next.control_r.position += Vec3::new(0.0, 0.0, -5.0) * move2abs;
                    next.control_r.orientation.rotate_x(-PI / 4.0 * move2abs);
                    next.control_l.position += Vec3::new(0.0, 0.0, -10.0) * move2abs;
                    next.main.position += Vec3::new(0.0, 0.0, -10.0) * move2abs;
                    next.control_l.orientation.rotate_x(-PI / 4.0 * move2abs);
                    next.main.orientation.rotate_x(-PI / 4.0 * move2abs);
                } else {
                    next.control_l.position += Vec3::new(5.0, 0.0, 5.0) * move1abs;
                    next.main.position += Vec3::new(3.0, 0.0, 5.0) * move1abs;
                    next.control_l.orientation.rotate_x(PI / 10.0 * move1abs);
                    next.main.orientation.rotate_x(PI / 10.0 * move1abs);
                    next.control_r.position += Vec3::new(-3.0, 0.0, 10.0) * move1abs;
                    next.control_r.orientation.rotate_x(PI / 4.0 * move1abs);

                    next.control_l.position += Vec3::new(0.0, 0.0, -5.0) * move2abs;
                    next.main.position += Vec3::new(0.0, 0.0, -5.0) * move2abs;
                    next.control_l.orientation.rotate_x(-PI / 4.0 * move2abs);
                    next.main.orientation.rotate_x(-PI / 4.0 * move2abs);
                    next.control_r.position += Vec3::new(0.0, 0.0, -10.0) * move2abs;
                    next.control_r.orientation.rotate_x(-PI / 4.0 * move2abs);
                }
            },
            _ => {
                next.chest.orientation = Quaternion::rotation_x(move2abs * -1.0)
                    * Quaternion::rotation_z(move1 * 1.2 + move2 * -1.8);
                next.hand_l.position = Vec3::new(-s_a.hand.0, s_a.hand.1, s_a.hand.2);
                next.hand_l.orientation = Quaternion::rotation_x(1.2);
                next.hand_r.position = Vec3::new(s_a.hand.0, s_a.hand.1, s_a.hand.2);
                next.hand_r.orientation = Quaternion::rotation_x(1.2);
            },
        }
        next
    }
}
