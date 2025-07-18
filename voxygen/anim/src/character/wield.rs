use super::{
    super::{Animation, vek::*},
    CharacterSkeleton, SkeletonAttr,
};
use common::{
    comp::item::{AbilitySpec, Hands, ToolKind},
    util::Dir,
};
use core::{f32::consts::PI, ops::Mul};

pub struct WieldAnimation;

type WieldAnimationDependency<'a> = (
    (Option<ToolKind>, Option<&'a AbilitySpec>),
    Option<ToolKind>,
    (Option<Hands>, Option<Hands>),
    Vec3<f32>,
    Vec3<f32>,
    Dir,
    Vec3<f32>,
    bool,
    f32,
);
impl Animation for WieldAnimation {
    type Dependency<'a> = WieldAnimationDependency<'a>;
    type Skeleton = CharacterSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"character_wield\0";

    #[cfg_attr(feature = "be-dyn-lib", unsafe(export_name = "character_wield"))]
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (
            (active_tool_kind, active_tool_spec),
            second_tool_kind,
            hands,
            orientation,
            last_ori,
            look_dir,
            velocity,
            is_riding,
            global_time,
        ): Self::Dependency<'_>,
        anim_time: f32,
        rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        *rate = 1.0;
        let lab: f32 = 0.8;
        let speed = Vec2::<f32>::from(velocity).magnitude();
        let speednorm = speed / 9.5;
        let mut next = (*skeleton).clone();
        let head_look = Vec2::new(
            (global_time + anim_time / 3.0).floor().mul(7331.0).sin() * 0.2,
            (global_time + anim_time / 3.0).floor().mul(1337.0).sin() * 0.1,
        );

        let beltstatic = (anim_time * 10.0 * lab + PI / 2.0).sin();
        let footvertlstatic = (anim_time * 10.0 * lab).sin();
        let footvertrstatic = (anim_time * 10.0 * lab + PI).sin();

        let slowalt = (anim_time * 9.0 + PI).cos();
        let u_slow = (anim_time * 4.5 + PI).sin();
        let slow = (anim_time * 7.0 + PI).sin();

        let u_slowalt = (anim_time * 5.0 + PI).cos();
        let direction = velocity.y * -0.098 * orientation.y + velocity.x * -0.098 * orientation.x;

        let ori: Vec2<f32> = Vec2::from(orientation);
        let last_ori = Vec2::from(last_ori);
        let tilt = (if vek::Vec2::new(ori, last_ori)
            .map(|o| o.magnitude_squared())
            .map(|m| m > 0.001 && m.is_finite())
            .reduce_and()
            && ori.angle_between(last_ori).is_finite()
        {
            ori.angle_between(last_ori).min(0.2)
                * last_ori.determine_side(Vec2::zero(), ori).signum()
        } else {
            0.0
        } * 1.25)
            * 4.0;
        let jump = if velocity.z == 0.0 { 0.0 } else { 1.0 };

        // next.second.scale = match hands {
        //     (Some(Hands::One), Some(Hands::One)) => Vec3::one(),
        //    (_, _) => Vec3::zero(),
        // };
        next.main.position = Vec3::new(0.0, 0.0, 0.0);
        next.main.orientation = Quaternion::rotation_z(0.0);
        next.main.scale = Vec3::one();
        next.second.position = Vec3::new(0.0, 0.0, 0.0);
        next.second.orientation = Quaternion::rotation_z(0.0);
        next.second.scale = Vec3::one();

        let is_moving = (speed > 0.2 && velocity.z == 0.0) || is_riding;

        if !is_moving {
            next.head.position = Vec3::new(0.0, s_a.head.0, s_a.head.1 + u_slow * 0.1);
            next.head.orientation = Quaternion::rotation_z(head_look.x + tilt * -0.75)
                * Quaternion::rotation_x(head_look.y.abs() + look_dir.z * 0.7);

            next.chest.position =
                Vec3::new(slowalt * 0.2, s_a.chest.0, s_a.chest.1 + u_slow * 0.35);
            next.belt.orientation = Quaternion::rotation_z(0.15 + beltstatic * tilt * 0.1);

            next.shorts.orientation = Quaternion::rotation_z(0.3 + beltstatic * tilt * 0.2);
            next.torso.orientation = Quaternion::rotation_z(tilt * 0.4);

            next.foot_l.position = Vec3::new(
                -s_a.foot.0,
                -2.0 + s_a.foot.1 + jump * -4.0,
                s_a.foot.2 + (tilt * footvertlstatic * 1.0).max(0.0),
            );
            next.foot_l.orientation = Quaternion::rotation_x(
                jump * -0.7 + u_slowalt * 0.035 + tilt * footvertlstatic * 0.1
                    - tilt.abs() * 0.3 * speednorm,
            ) * Quaternion::rotation_z(-tilt * 0.3);

            next.foot_r.position = Vec3::new(
                s_a.foot.0,
                2.0 + s_a.foot.1 + jump * 4.0,
                s_a.foot.2 + (tilt * footvertrstatic * 1.0).max(0.0),
            );
            next.foot_r.orientation = Quaternion::rotation_x(
                jump * 0.7 + u_slow * 0.035 + tilt * footvertrstatic * 0.1
                    - tilt.abs() * 0.3 * speednorm,
            ) * Quaternion::rotation_z(-tilt * 0.3);

            next.chest.orientation = Quaternion::rotation_y(u_slowalt * 0.04)
                * Quaternion::rotation_z(0.15 + tilt * -0.4);

            next.belt.position = Vec3::new(0.0, s_a.belt.0, s_a.belt.1);

            // next.back.orientation = Quaternion::rotation_x(-0.2);
            next.shorts.position = Vec3::new(0.0, s_a.shorts.0, s_a.shorts.1);
        }
        match (hands, active_tool_kind, second_tool_kind) {
            ((Some(Hands::Two), _), tool, _) | ((None, Some(Hands::Two)), _, tool) => match tool {
                Some(ToolKind::Sword) => {
                    next.control_l.position = next.hand_l.position * 0.2
                        + Vec3::new(
                            s_a.sc.0,
                            s_a.sc.1 - slow * 2.0 * speednorm,
                            s_a.sc.2 + direction * -5.0 - slow * 2.0 * speednorm,
                        );
                    next.control_r.position = next.control_l.position;

                    next.hand_l.position = Vec3::new(s_a.shl.0 - 0.5, s_a.shl.1, s_a.shl.2);
                    next.hand_l.orientation =
                        Quaternion::rotation_x(s_a.shl.3) * Quaternion::rotation_y(s_a.shl.4);
                    next.control_l.orientation = Quaternion::rotation_x(s_a.sc.3 + u_slow * 0.05)
                        * Quaternion::rotation_z(u_slowalt * 0.04);
                    next.control_r.orientation = Quaternion::rotation_x(s_a.sc.3 + u_slow * 0.15)
                        * Quaternion::rotation_z(u_slowalt * 0.08);
                    next.hand_r.position = Vec3::zero();
                    next.hand_r.orientation =
                        next.hand_l.orientation * Quaternion::rotation_y(PI * 0.3);
                },
                Some(ToolKind::Axe) => {
                    next.main.position = Vec3::new(0.0, 0.0, 0.0);
                    next.main.orientation = Quaternion::rotation_x(0.0);

                    if speed < 0.5 {
                        next.head.position =
                            Vec3::new(0.0, 0.0 + s_a.head.0, s_a.head.1 + u_slow * 0.1);
                        next.head.orientation = Quaternion::rotation_z(head_look.x)
                            * Quaternion::rotation_x(0.15 + head_look.y.abs() + look_dir.z * 0.7);
                        next.chest.orientation = Quaternion::rotation_x(-0.15)
                            * Quaternion::rotation_y(u_slowalt * 0.04)
                            * Quaternion::rotation_z(0.15);
                        next.belt.position = Vec3::new(0.0, 1.0 + s_a.belt.0, s_a.belt.1);
                        next.belt.orientation = Quaternion::rotation_x(0.15)
                            * Quaternion::rotation_y(u_slowalt * 0.03)
                            * Quaternion::rotation_z(0.15);
                        next.shorts.position = Vec3::new(0.0, 1.0 + s_a.shorts.0, s_a.shorts.1);
                        next.shorts.orientation =
                            Quaternion::rotation_x(0.15) * Quaternion::rotation_z(0.25);
                    }
                    next.hand_l.position = Vec3::new(s_a.ahl.0, s_a.ahl.1, s_a.ahl.2);
                    next.hand_l.orientation =
                        Quaternion::rotation_x(s_a.ahl.3) * Quaternion::rotation_y(s_a.ahl.4);
                    next.hand_r.position = Vec3::new(s_a.ahr.0, s_a.ahr.1, s_a.ahr.2);
                    next.hand_r.orientation =
                        Quaternion::rotation_x(s_a.ahr.3) * Quaternion::rotation_z(PI);

                    next.control.position =
                        Vec3::new(s_a.ac.0, s_a.ac.1, s_a.ac.2 + direction * -5.0);
                    next.control.orientation = Quaternion::rotation_x(s_a.ac.3)
                        * Quaternion::rotation_y(s_a.ac.4)
                        * Quaternion::rotation_z(s_a.ac.5);
                },
                Some(ToolKind::Hammer | ToolKind::Pick) => {
                    next.hand_l.position = Vec3::new(s_a.hhl.0, s_a.hhl.1 + 3.0, s_a.hhl.2 - 1.0);
                    next.hand_l.orientation = Quaternion::rotation_x(s_a.hhl.3)
                        * Quaternion::rotation_y(s_a.hhl.4)
                        * Quaternion::rotation_z(s_a.hhl.5);
                    next.hand_r.position = Vec3::new(s_a.hhr.0, s_a.hhr.1 + 3.0, s_a.hhr.2 + 1.0);
                    next.hand_r.orientation = Quaternion::rotation_x(s_a.hhr.3)
                        * Quaternion::rotation_y(s_a.hhr.4)
                        * Quaternion::rotation_z(s_a.hhr.5);

                    next.control.position =
                        Vec3::new(s_a.hc.0 - 1.0, s_a.hc.1, s_a.hc.2 + direction * -5.0 - 3.0);
                    next.control.orientation = Quaternion::rotation_x(s_a.hc.3 + u_slow * 0.15)
                        * Quaternion::rotation_y(s_a.hc.4)
                        * Quaternion::rotation_z(s_a.hc.5 + u_slowalt * 0.07);
                },
                Some(ToolKind::Staff) | Some(ToolKind::Sceptre) => {
                    next.control_l.position = next.hand_l.position * 0.2
                        + Vec3::new(
                            s_a.sc.0 + 1.0,
                            s_a.sc.1 - slow * 2.0 * speednorm - 3.0,
                            s_a.sc.2 + direction * -5.0 - slow * 2.0 * speednorm - 3.0,
                        );
                    next.control_r.position = next.control_l.position;

                    next.hand_l.position = Vec3::new(s_a.shl.0 - 0.5, s_a.shl.1, s_a.shl.2 + 0.0);
                    next.hand_l.orientation =
                        Quaternion::rotation_x(s_a.shl.3) * Quaternion::rotation_y(s_a.shl.4);
                    next.control_l.orientation = Quaternion::rotation_x(s_a.sc.3 + u_slow * 0.05)
                        * Quaternion::rotation_z(u_slowalt * 0.04);
                    next.control_r.orientation = Quaternion::rotation_x(s_a.sc.3 + u_slow * 0.15)
                        * Quaternion::rotation_z(u_slowalt * 0.08);
                    next.hand_r.position = Vec3::new(0.0, 0.0, 8.0);
                    next.hand_r.orientation =
                        next.hand_l.orientation * Quaternion::rotation_y(PI * 0.3);
                },
                Some(ToolKind::Bow) => {
                    next.main.position = Vec3::new(0.0, 0.0, 0.0);
                    next.main.orientation = Quaternion::rotation_x(0.0);
                    next.hand_l.position = Vec3::new(s_a.bhl.0, s_a.bhl.1, s_a.bhl.2);
                    next.hand_l.orientation = Quaternion::rotation_x(s_a.bhl.3);
                    next.hand_r.position = Vec3::new(s_a.bhr.0, s_a.bhr.1, s_a.bhr.2);
                    next.hand_r.orientation = Quaternion::rotation_x(s_a.bhr.3);

                    next.hold.position = Vec3::new(0.0, -1.0, -5.2);
                    next.hold.orientation = Quaternion::rotation_x(-PI / 2.0);
                    next.hold.scale = Vec3::one() * 1.0;

                    next.control.position =
                        Vec3::new(s_a.bc.0, s_a.bc.1, s_a.bc.2 + direction * -5.0);
                    next.control.orientation = Quaternion::rotation_x(u_slow * 0.06)
                        * Quaternion::rotation_y(s_a.bc.4)
                        * Quaternion::rotation_z(s_a.bc.5 + u_slowalt * 0.1);
                },
                Some(ToolKind::Debug) => {
                    next.hand_l.position = Vec3::new(-7.0, 4.0, 3.0);
                    next.hand_l.orientation = Quaternion::rotation_x(1.27);
                    next.main.position = Vec3::new(-5.0, 5.0, 23.0);
                    next.main.orientation = Quaternion::rotation_x(PI);
                },
                Some(ToolKind::Farming) => {
                    if speed < 0.5 {
                        next.head.orientation = Quaternion::rotation_z(head_look.x)
                            * Quaternion::rotation_x(-0.2 + head_look.y.abs() + look_dir.z * 0.7);
                    }
                    next.hand_l.position = Vec3::new(9.0, 1.0, 1.0);
                    next.hand_l.orientation = Quaternion::rotation_x(PI / 2.0);
                    next.hand_r.position = Vec3::new(9.0, 1.0, 11.0);
                    next.hand_r.orientation = Quaternion::rotation_x(PI / 2.0);
                    next.main.position = Vec3::new(7.5, 7.5, 13.2);
                    next.main.orientation = Quaternion::rotation_y(PI);

                    next.control.position = Vec3::new(-11.0 + slow * 2.0, 1.8, 4.0);
                    next.control.orientation = Quaternion::rotation_x(u_slow * 0.1)
                        * Quaternion::rotation_y(0.6 + u_slow * 0.1)
                        * Quaternion::rotation_z(u_slowalt * 0.1);
                },
                Some(ToolKind::Shovel) => {
                    next.hand_l.position = Vec3::new(8.0, 6.0, 3.0);
                    next.hand_l.orientation = Quaternion::rotation_x(PI / 2.0);
                    next.hand_r.position = Vec3::new(8.0, 6.0, 15.0);
                    next.hand_r.orientation = Quaternion::rotation_x(PI / 2.0);
                    next.main.position = Vec3::new(7.5, 7.5, 13.2);
                    next.main.orientation = Quaternion::rotation_y(PI);

                    next.control.position = Vec3::new(-11.0 + slow * 0.02, 1.8, 4.0);
                    next.control.orientation = Quaternion::rotation_x(u_slow * 0.01)
                        * Quaternion::rotation_y(0.8 + u_slow * 0.01)
                        * Quaternion::rotation_z(u_slowalt * 0.01);
                },
                Some(ToolKind::Instrument) => {
                    if let Some(AbilitySpec::Custom(spec)) = active_tool_spec {
                        match spec.as_str() {
                            "Lyre" | "IcyTalharpa" | "WildskinDrum" | "Steeltonguedrum" => {
                                if speed < 0.5 {
                                    next.head.orientation = Quaternion::rotation_z(head_look.x)
                                        * Quaternion::rotation_x(
                                            0.0 + head_look.y.abs() + look_dir.z * 0.7,
                                        );
                                }
                                next.hand_l.position = Vec3::new(0.0, 2.0, -4.0);
                                next.hand_l.orientation =
                                    Quaternion::rotation_x(PI / 2.0) * Quaternion::rotation_z(PI);
                                next.hand_r.position = Vec3::new(-4.0, 2.0, 6.0);
                                next.hand_r.orientation = Quaternion::rotation_x(PI / 2.0)
                                    * Quaternion::rotation_z(PI / 2.0);
                                next.main.position = Vec3::new(-2.0, 10.0, 12.0);
                                next.main.orientation = Quaternion::rotation_y(PI);

                                next.control.position = Vec3::new(-2.0 + slow * 0.5, 0.5, 0.8);
                                next.control.orientation = Quaternion::rotation_x(u_slow * 0.1)
                                    * Quaternion::rotation_y(2.0 + u_slow * 0.1)
                                    * Quaternion::rotation_z(u_slowalt * 0.1);
                            },
                            "Flute" | "GlassFlute" => {
                                if speed < 0.5 {
                                    next.head.orientation = Quaternion::rotation_z(head_look.x)
                                        * Quaternion::rotation_x(
                                            0.0 + head_look.y.abs() + look_dir.z * 0.7,
                                        );
                                }
                                next.hand_l.position = Vec3::new(-1.0, 4.0, -1.0);
                                next.hand_l.orientation = Quaternion::rotation_x(2.5)
                                    * Quaternion::rotation_y(0.9)
                                    * Quaternion::rotation_z(PI);
                                next.hand_r.position = Vec3::new(-4.0, 2.0, 6.0);
                                next.hand_r.orientation = Quaternion::rotation_x(PI / 2.0);
                                next.main.position = Vec3::new(12.0, 3.0, 4.0);
                                next.main.orientation =
                                    Quaternion::rotation_x(PI) * Quaternion::rotation_y(-1.2);

                                next.control.position = Vec3::new(-2.0 + slow * 0.5, 0.5, 0.8);
                                next.control.orientation = Quaternion::rotation_x(u_slow * 0.1)
                                    * Quaternion::rotation_y(2.0 + u_slow * 0.1)
                                    * Quaternion::rotation_z(u_slowalt * 0.1);
                            },
                            "DoubleBass" => {
                                if speed < 0.5 {
                                    next.head.orientation = Quaternion::rotation_z(head_look.x)
                                        * Quaternion::rotation_x(
                                            0.0 + head_look.y.abs() + look_dir.z * 0.7,
                                        );
                                }
                                next.hand_l.position = Vec3::new(-6.0, 6.0, -5.0);
                                next.hand_l.orientation = Quaternion::rotation_x((PI / 2.0) + 0.3)
                                    * Quaternion::rotation_y(0.7)
                                    * Quaternion::rotation_y(0.25)
                                    * Quaternion::rotation_z(PI);
                                next.hand_r.position = Vec3::new(-2.0, 4.0, 5.0);
                                next.hand_r.orientation = Quaternion::rotation_x(PI / 2.0)
                                    * Quaternion::rotation_z(PI / 2.0);
                                next.main.position = Vec3::new(-14.0, 6.0, -6.0);
                                next.main.orientation = Quaternion::rotation_x(-0.2)
                                    * Quaternion::rotation_y(1.2)
                                    * Quaternion::rotation_z(-1.2);

                                next.control.position = Vec3::new(-2.0 + slow * 0.5, 0.5, 0.8);
                                next.control.orientation = Quaternion::rotation_x(u_slow * 0.1)
                                    * Quaternion::rotation_y(2.0 + u_slow * 0.1)
                                    * Quaternion::rotation_z(u_slowalt * 0.1);
                            },
                            "Kora" => {
                                if speed < 0.5 {
                                    next.head.orientation = Quaternion::rotation_z(head_look.x)
                                        * Quaternion::rotation_x(
                                            0.0 + head_look.y.abs() + look_dir.z * 0.7,
                                        );
                                }
                                next.hand_l.position = Vec3::new(-6.0, 6.0, -5.0);
                                next.hand_l.orientation = Quaternion::rotation_x((PI / 2.0) + 0.3)
                                    * Quaternion::rotation_y(0.7)
                                    * Quaternion::rotation_y(0.25)
                                    * Quaternion::rotation_z(PI);
                                next.hand_r.position = Vec3::new(-2.0, 4.0, 5.0);
                                next.hand_r.orientation = Quaternion::rotation_x(PI / 2.0)
                                    * Quaternion::rotation_z(PI / 2.0);
                                next.main.position = Vec3::new(-14.0, 6.0, -6.0);
                                next.main.orientation = Quaternion::rotation_x(-0.2)
                                    * Quaternion::rotation_y(1.2)
                                    * Quaternion::rotation_z(1.3);

                                next.control.position = Vec3::new(-2.0 + slow * 0.5, 0.5, 0.8);
                                next.control.orientation = Quaternion::rotation_x(u_slow * 0.1)
                                    * Quaternion::rotation_y(2.0 + u_slow * 0.1)
                                    * Quaternion::rotation_z(u_slowalt * 0.1);
                            },
                            "Washboard" | "TimbrelOfChaos" | "Rhythmo" | "StarlightConch" => {
                                if speed < 0.5 {
                                    next.head.orientation = Quaternion::rotation_z(head_look.x)
                                        * Quaternion::rotation_x(
                                            0.0 + head_look.y.abs() + look_dir.z * 0.7,
                                        );
                                }
                                next.hand_l.position = Vec3::new(0.0, 2.0, -4.0);
                                next.hand_l.orientation =
                                    Quaternion::rotation_x(PI / 2.0) * Quaternion::rotation_z(PI);
                                next.hand_r.position = Vec3::new(-4.0, 2.0, 6.0);
                                next.hand_r.orientation = Quaternion::rotation_x(PI / 2.0)
                                    * Quaternion::rotation_z(PI / 2.0);
                                next.main.position = Vec3::new(-2.0, 10.0, 12.0);
                                next.main.orientation = Quaternion::rotation_y(PI);

                                next.control.position = Vec3::new(-2.0 + slow * 0.5, 0.5, 0.8);
                                next.control.orientation = Quaternion::rotation_x(u_slow * 0.1)
                                    * Quaternion::rotation_y(2.0 + u_slow * 0.1)
                                    * Quaternion::rotation_z(u_slowalt * 0.1);
                            },
                            "Kalimba" => {
                                if speed < 0.5 {
                                    next.head.orientation = Quaternion::rotation_z(head_look.x)
                                        * Quaternion::rotation_x(
                                            0.0 + head_look.y.abs() + look_dir.z * 0.7,
                                        );
                                }
                                next.hand_l.position = Vec3::new(0.0, 2.0, -4.0);
                                next.hand_l.orientation =
                                    Quaternion::rotation_x(PI / 2.0) * Quaternion::rotation_z(PI);
                                next.hand_r.position = Vec3::new(-4.0, 2.0, 6.0);
                                next.hand_r.orientation = Quaternion::rotation_x(PI / 2.0)
                                    * Quaternion::rotation_z(PI / 2.0);
                                next.main.position = Vec3::new(0.0, 7.0, 12.0);
                                next.main.orientation =
                                    Quaternion::rotation_y(PI) * Quaternion::rotation_z(PI);

                                next.control.position = Vec3::new(-2.0 + slow * 0.5, 0.5, 0.8);
                                next.control.orientation = Quaternion::rotation_x(u_slow * 0.1)
                                    * Quaternion::rotation_y(2.0 + u_slow * 0.1)
                                    * Quaternion::rotation_z(u_slowalt * 0.1);
                            },
                            "Lute" | "Shamisen" | "Banjo" | "Oud" => {
                                if speed < 0.5 {
                                    next.head.orientation = Quaternion::rotation_z(head_look.x)
                                        * Quaternion::rotation_x(
                                            0.0 + head_look.y.abs() + look_dir.z * 0.7,
                                        );
                                }
                                next.hand_l.position = Vec3::new(-2.0, 5.0, -5.0);
                                next.hand_l.orientation = Quaternion::rotation_x((PI / 2.0) + 0.3)
                                    * Quaternion::rotation_y(0.7)
                                    * Quaternion::rotation_y(0.25)
                                    * Quaternion::rotation_z(PI);
                                next.hand_r.position = Vec3::new(-5.0, 2.0, 6.0);
                                next.hand_r.orientation = Quaternion::rotation_x(PI / 2.0)
                                    * Quaternion::rotation_z(PI / 2.0);
                                next.main.position = Vec3::new(-2.0, 4.0, -12.0);
                                next.main.orientation = Quaternion::rotation_x(0.0)
                                    * Quaternion::rotation_y(0.2)
                                    * Quaternion::rotation_z(-1.3);

                                next.control.position = Vec3::new(-2.0 + slow * 0.5, 0.5, 0.8);
                                next.control.orientation = Quaternion::rotation_x(u_slow * 0.1)
                                    * Quaternion::rotation_y(2.0 + u_slow * 0.1)
                                    * Quaternion::rotation_z(u_slowalt * 0.1);
                            },
                            "ViolaPizzicato" => {
                                if speed < 0.5 {
                                    next.head.orientation = Quaternion::rotation_z(head_look.x)
                                        * Quaternion::rotation_x(
                                            0.0 + head_look.y.abs() + look_dir.z * 0.7,
                                        );
                                }
                                next.hand_l.position = Vec3::new(-2.0, 5.0, -5.0);
                                next.hand_l.orientation = Quaternion::rotation_x((PI / 2.0) + 0.3)
                                    * Quaternion::rotation_y(0.7)
                                    * Quaternion::rotation_y(0.25)
                                    * Quaternion::rotation_z(PI);
                                next.hand_r.position = Vec3::new(-5.0, 2.0, 6.0);
                                next.hand_r.orientation = Quaternion::rotation_x(PI / 2.0)
                                    * Quaternion::rotation_z(PI / 2.0);
                                next.main.position = Vec3::new(-2.0, 6.0, -12.0);
                                next.main.orientation = Quaternion::rotation_x(0.0)
                                    * Quaternion::rotation_y(0.2)
                                    * Quaternion::rotation_z(-1.3);

                                next.control.position = Vec3::new(-2.0 + slow * 0.5, 0.5, 0.8);
                                next.control.orientation = Quaternion::rotation_x(u_slow * 0.1)
                                    * Quaternion::rotation_y(2.0 + u_slow * 0.1)
                                    * Quaternion::rotation_z(u_slowalt * 0.1);
                            },
                            "Guitar" | "DarkGuitar" => {
                                if speed < 0.5 {
                                    next.head.orientation = Quaternion::rotation_z(head_look.x)
                                        * Quaternion::rotation_x(
                                            0.0 + head_look.y.abs() + look_dir.z * 0.7,
                                        );
                                }
                                next.hand_l.position = Vec3::new(0.0, 5.0, -4.0);
                                next.hand_l.orientation = Quaternion::rotation_x((PI / 2.0) + 0.3)
                                    * Quaternion::rotation_y(0.7)
                                    * Quaternion::rotation_y(0.25)
                                    * Quaternion::rotation_z(PI);
                                next.hand_r.position = Vec3::new(-5.0, 2.0, 6.0);
                                next.hand_r.orientation = Quaternion::rotation_x(PI / 2.0)
                                    * Quaternion::rotation_z(PI / 2.0);
                                next.main.position = Vec3::new(-2.0, 4.0, -12.0);
                                next.main.orientation = Quaternion::rotation_x(0.0)
                                    * Quaternion::rotation_y(0.2)
                                    * Quaternion::rotation_z(-1.3);

                                next.control.position = Vec3::new(-2.0 + slow * 0.5, 0.5, 0.8);
                                next.control.orientation = Quaternion::rotation_x(u_slow * 0.1)
                                    * Quaternion::rotation_y(2.0 + u_slow * 0.1)
                                    * Quaternion::rotation_z(u_slowalt * 0.1);
                            },
                            "Melodica" => {
                                if speed < 0.5 {
                                    next.head.orientation = Quaternion::rotation_z(head_look.x)
                                        * Quaternion::rotation_x(
                                            0.0 + head_look.y.abs() + look_dir.z * 0.7,
                                        );
                                }
                                next.hand_l.position = Vec3::new(-1.0, 3.0, -2.0);
                                next.hand_l.orientation = Quaternion::rotation_x(2.0)
                                    * Quaternion::rotation_z(-0.5)
                                    * Quaternion::rotation_y(0.4)
                                    * Quaternion::rotation_z(PI);
                                next.hand_r.position = Vec3::new(-4.0, 2.0, 6.0);
                                next.hand_r.orientation = Quaternion::rotation_x(1.2)
                                    * Quaternion::rotation_y(-0.3)
                                    * Quaternion::rotation_z(1.5);
                                next.main.position = Vec3::new(-14.0, 3.0, -6.0);
                                next.main.orientation = Quaternion::rotation_x(0.0)
                                    * Quaternion::rotation_y(1.1)
                                    * Quaternion::rotation_z(-1.5);

                                next.control.position = Vec3::new(-2.0 + slow * 0.5, 0.5, 0.8);
                                next.control.orientation = Quaternion::rotation_x(u_slow * 0.1)
                                    * Quaternion::rotation_y(2.0 + u_slow * 0.1)
                                    * Quaternion::rotation_z(u_slowalt * 0.1);
                            },
                            "Sitar" => {
                                if speed < 0.5 {
                                    next.head.orientation = Quaternion::rotation_z(head_look.x)
                                        * Quaternion::rotation_x(
                                            0.0 + head_look.y.abs() + look_dir.z * 0.7,
                                        );
                                }
                                next.hand_l.position = Vec3::new(-1.0, 5.0, -2.5);
                                next.hand_l.orientation = Quaternion::rotation_x((PI / 2.0) + 0.3)
                                    * Quaternion::rotation_y(0.2)
                                    * Quaternion::rotation_y(0.25)
                                    * Quaternion::rotation_z(PI);
                                next.hand_r.position = Vec3::new(-5.0, 2.0, 6.0);
                                next.hand_r.orientation = Quaternion::rotation_x(PI / 2.0)
                                    * Quaternion::rotation_z(PI / 2.0);
                                next.main.position = Vec3::new(-2.0, 4.0, -12.0);
                                next.main.orientation = Quaternion::rotation_x(0.0)
                                    * Quaternion::rotation_y(0.2)
                                    * Quaternion::rotation_z(-1.3);

                                next.control.position = Vec3::new(-2.0 + slow * 0.5, 0.5, 0.8);
                                next.control.orientation = Quaternion::rotation_x(u_slow * 0.1)
                                    * Quaternion::rotation_y(2.0 + u_slow * 0.1)
                                    * Quaternion::rotation_z(u_slowalt * 0.1);
                            },
                            _ => {},
                        }
                    }
                },
                Some(ToolKind::Shield) => {
                    next.hand_l.position = Vec3::new(0.0, -2.0, 0.0);
                    next.hand_l.orientation = Quaternion::rotation_x(PI / 2.0);

                    next.hand_r.position = Vec3::new(0.0, 0.0, 0.0);
                    next.hand_r.orientation =
                        Quaternion::rotation_x(PI / 2.0) * Quaternion::rotation_y(2.0);

                    next.control.position = Vec3::new(0.0, 7.0, 4.0);
                    next.control.orientation =
                        Quaternion::rotation_y(-0.5) * Quaternion::rotation_z(-1.25);
                },
                _ => {},
            },
            ((_, _), _, _) => {},
        };
        match hands {
            (Some(Hands::One), _) => {
                next.control_l.position =
                    next.hand_l.position * Vec3::new(0.5, 0.5, 0.3) + Vec3::new(-4.0, 0.0, 0.0);
                next.control_l.orientation = Quaternion::lerp(
                    next.hand_l.orientation,
                    Quaternion::rotation_x(PI * -0.5),
                    0.65,
                );
                next.hand_l.position = Vec3::new(0.0, -2.0, 0.0);
                next.hand_l.orientation = Quaternion::rotation_x(PI * 0.5);
            },
            (_, _) => {},
        };
        match hands {
            (None | Some(Hands::One), Some(Hands::One)) => {
                next.control_r.position =
                    next.hand_r.position * Vec3::new(0.5, 0.5, 0.3) + Vec3::new(4.0, 0.0, 0.0);
                next.control_r.orientation = Quaternion::lerp(
                    next.hand_r.orientation,
                    Quaternion::rotation_x(PI * -0.5),
                    0.65,
                );
                next.hand_r.position = Vec3::new(0.0, -2.0, 0.0);
                next.hand_r.orientation = Quaternion::rotation_x(PI * 0.5);
            },
            (_, _) => {},
        };
        match hands {
            (None, None) | (None, Some(Hands::One)) => {
                next.hand_l.position = Vec3::new(-8.0, 2.0, 1.0);
                next.hand_l.orientation =
                    Quaternion::rotation_x(0.5) * Quaternion::rotation_y(0.25);
            },
            (_, _) => {},
        };
        match hands {
            (None, None) | (Some(Hands::One), None) => {
                // next.hand_r.position = Vec3::new(8.0, 2.0, 1.0);
                // next.hand_r.orientation =
                //     Quaternion::rotation_x(0.5) *
                // Quaternion::rotation_y(-0.25);
            },
            (_, _) => {},
        };

        if let (None, Some(Hands::Two)) = hands {
            next.second = next.main;
        }

        next.do_hold_lantern(s_a, anim_time, anim_time, speednorm, 0.0, tilt);

        next
    }
}
