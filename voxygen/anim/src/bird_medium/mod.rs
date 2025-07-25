pub mod alpha;
pub mod breathe;
pub mod dash;
pub mod feed;
pub mod fly;
pub mod idle;
pub mod run;
pub mod shockwave;
pub mod shoot;
pub mod stunned;
pub mod summon;
pub mod swim;

// Reexports
pub use self::{
    alpha::AlphaAnimation, breathe::BreatheAnimation, dash::DashAnimation, feed::FeedAnimation,
    fly::FlyAnimation, idle::IdleAnimation, run::RunAnimation, shockwave::ShockwaveAnimation,
    shoot::ShootAnimation, stunned::StunnedAnimation, summon::SummonAnimation, swim::SwimAnimation,
};

use super::{FigureBoneData, Skeleton, vek::*};
use common::comp::{self};
use core::convert::TryFrom;

pub type Body = comp::bird_medium::Body;

skeleton_impls!(struct BirdMediumSkeleton ComputedBirdMediumSkeleton {
    + head
    + chest
    + tail
    + wing_in_l
    + wing_in_r
    + wing_out_l
    + wing_out_r
    + leg_l
    + leg_r
});

impl Skeleton for BirdMediumSkeleton {
    type Attr = SkeletonAttr;
    type Body = Body;
    type ComputedSkeleton = ComputedBirdMediumSkeleton;

    const BONE_COUNT: usize = ComputedBirdMediumSkeleton::BONE_COUNT;
    #[cfg(feature = "use-dyn-lib")]
    const COMPUTE_FN: &'static [u8] = b"bird_medium_compute_mats\0";

    #[cfg_attr(
        feature = "be-dyn-lib",
        unsafe(export_name = "bird_medium_compute_mats")
    )]

    fn compute_matrices_inner(
        &self,
        base_mat: Mat4<f32>,
        buf: &mut [FigureBoneData; super::MAX_BONE_COUNT],
        body: Self::Body,
    ) -> Self::ComputedSkeleton {
        let base_mat = base_mat * Mat4::scaling_3d(SkeletonAttr::from(&body).scaler / 8.0);

        let chest_mat = base_mat * Mat4::<f32>::from(self.chest);
        let head_mat = chest_mat * Mat4::<f32>::from(self.head);
        let tail_mat = chest_mat * Mat4::<f32>::from(self.tail);
        let wing_in_l_mat = chest_mat * Mat4::<f32>::from(self.wing_in_l);
        let wing_in_r_mat = chest_mat * Mat4::<f32>::from(self.wing_in_r);
        let wing_out_l_mat = wing_in_l_mat * Mat4::<f32>::from(self.wing_out_l);
        let wing_out_r_mat = wing_in_r_mat * Mat4::<f32>::from(self.wing_out_r);
        let leg_l_mat = base_mat * Mat4::<f32>::from(self.leg_l);
        let leg_r_mat = base_mat * Mat4::<f32>::from(self.leg_r);

        let computed_skeleton = ComputedBirdMediumSkeleton {
            head: head_mat,
            chest: chest_mat,
            tail: tail_mat,
            wing_in_l: wing_in_l_mat,
            wing_in_r: wing_in_r_mat,
            wing_out_l: wing_out_l_mat,
            wing_out_r: wing_out_r_mat,
            leg_l: leg_l_mat,
            leg_r: leg_r_mat,
        };

        computed_skeleton.set_figure_bone_data(buf);
        computed_skeleton
    }
}

pub struct SkeletonAttr {
    head: (f32, f32),
    chest: (f32, f32),
    tail: (f32, f32),
    wing_in: (f32, f32, f32),
    wing_out: (f32, f32, f32),
    leg: (f32, f32, f32),
    scaler: f32,
    feed: f32,
}

impl<'a> TryFrom<&'a comp::Body> for SkeletonAttr {
    type Error = ();

    fn try_from(body: &'a comp::Body) -> Result<Self, Self::Error> {
        match body {
            comp::Body::BirdMedium(body) => Ok(SkeletonAttr::from(body)),
            _ => Err(()),
        }
    }
}

impl Default for SkeletonAttr {
    fn default() -> Self {
        Self {
            chest: (0.0, 0.0),
            head: (0.0, 0.0),
            tail: (0.0, 0.0),
            wing_in: (0.0, 0.0, 0.0),
            wing_out: (0.0, 0.0, 0.0),
            leg: (0.0, 0.0, 0.0),
            scaler: 0.0,
            feed: 0.0,
        }
    }
}

impl<'a> From<&'a Body> for SkeletonAttr {
    fn from(body: &'a Body) -> Self {
        use comp::bird_medium::{BodyType::*, Species::*};
        Self {
            chest: match (body.species, body.body_type) {
                (SnowyOwl, _) => (0.0, 4.5),
                (HornedOwl, _) => (0.0, 4.5),
                (Duck, _) => (0.0, 4.0),
                (Cockatiel, _) => (0.0, 4.0),
                (Chicken, Male) => (0.0, 5.5),
                (Chicken, Female) => (0.0, 5.5),
                (Bat, _) => (0.0, 7.0),
                (Penguin, _) => (0.0, 7.0),
                (Goose, _) => (0.0, 6.5),
                (Peacock, _) => (0.0, 7.5),
                (Eagle, _) => (0.0, 6.0),
                (Parrot, _) => (0.0, 5.0),
                (Crow, _) => (0.0, 4.0),
                (Dodo, _) => (0.0, 6.0),
                (Parakeet, _) => (0.0, 3.5),
                (Puffin, _) => (0.0, 6.0),
                (Toucan, _) => (0.0, 5.0),
                (BloodmoonBat, _) => (0.0, 7.0),
                (VampireBat, _) => (0.0, 7.5),
            },
            head: match (body.species, body.body_type) {
                (SnowyOwl, _) => (3.5, 5.0),
                (HornedOwl, _) => (3.5, 5.0),
                (Duck, _) => (2.0, 5.5),
                (Cockatiel, _) => (3.0, 5.5),
                (Chicken, Male) => (3.0, 4.5),
                (Chicken, Female) => (3.0, 6.0),
                (Bat, _) => (2.5, 5.0),
                (Penguin, _) => (1.5, 6.0),
                (Goose, _) => (5.0, 4.0),
                (Peacock, _) => (3.0, 5.0),
                (Eagle, _) => (4.5, 5.0),
                (Parrot, _) => (1.5, 4.5),
                (Crow, _) => (4.5, 4.0),
                (Dodo, _) => (3.5, 4.5),
                (Parakeet, _) => (2.0, 4.0),
                (Puffin, _) => (3.5, 5.5),
                (Toucan, _) => (2.5, 4.5),
                (BloodmoonBat, _) => (4.0, 5.0),
                (VampireBat, _) => (2.5, 5.0),
            },
            tail: match (body.species, body.body_type) {
                (SnowyOwl, _) => (-6.0, -2.0),
                (HornedOwl, _) => (-6.0, -2.0),
                (Duck, _) => (-5.0, 1.0),
                (Cockatiel, _) => (-3.0, -0.5),
                (Chicken, Male) => (-7.5, 3.5),
                (Chicken, Female) => (-4.5, 3.0),
                (Bat, _) => (-8.0, -4.0),
                (Penguin, _) => (-3.0, -4.0),
                (Goose, _) => (-4.0, 3.0),
                (Peacock, _) => (-5.5, 1.0),
                (Eagle, _) => (-6.0, -2.0),
                (Parrot, _) => (-6.0, 0.0),
                (Crow, _) => (-5.0, -1.5),
                (Dodo, _) => (-7.5, -0.5),
                (Parakeet, _) => (-5.0, -1.0),
                (Puffin, _) => (-7.0, -2.0),
                (Toucan, _) => (-6.0, 0.0),
                (BloodmoonBat, _) => (-6.0, 1.0),
                (VampireBat, _) => (-5.0, -4.0),
            },
            wing_in: match (body.species, body.body_type) {
                (SnowyOwl, _) => (2.5, 1.0, 1.5),
                (HornedOwl, _) => (2.5, 1.0, 1.5),
                (Duck, _) => (2.5, 0.5, 1.0),
                (Cockatiel, _) => (1.5, 0.5, 1.0),
                (Chicken, Male) => (2.0, 1.0, 1.0),
                (Chicken, Female) => (2.0, 1.5, 1.0),
                (Bat, _) => (2.0, 2.0, -2.0),
                (Penguin, _) => (3.0, 0.5, 1.0),
                (Goose, _) => (2.5, 1.0, 2.0),
                (Peacock, _) => (2.0, 1.0, 1.0),
                (Eagle, _) => (3.0, 1.5, 1.0),
                (Parrot, _) => (2.0, 0.5, 0.0),
                (Crow, _) => (2.0, 0.5, 1.0),
                (Dodo, _) => (3.0, -1.0, 1.0),
                (Parakeet, _) => (1.0, 0.5, 0.0),
                (Puffin, _) => (2.0, 0.0, 1.0),
                (Toucan, _) => (2.0, 0.5, 0.0),
                (BloodmoonBat, _) => (4.0, 3.0, 1.5),
                (VampireBat, _) => (2.0, 2.0, -2.0),
            },
            wing_out: match (body.species, body.body_type) {
                (SnowyOwl, _) => (4.5, 3.5, 1.0),
                (HornedOwl, _) => (4.5, 3.5, 1.0),
                (Duck, _) => (3.0, 2.0, 0.5),
                (Cockatiel, _) => (3.0, 2.0, 0.5),
                (Chicken, Male) => (3.0, 2.0, 0.5),
                (Chicken, Female) => (3.0, 2.0, 0.5),
                (Bat, _) => (5.0, 3.0, 0.0),
                (Penguin, _) => (3.0, 2.5, 0.5),
                (Goose, _) => (4.0, 3.0, 0.5),
                (Peacock, _) => (5.0, 3.0, 0.5),
                (Eagle, _) => (8.0, 4.5, 0.5),
                (Parrot, _) => (5.0, 3.0, 0.5),
                (Crow, _) => (5.0, 3.0, 0.5),
                (Dodo, _) => (3.0, 3.0, 0.5),
                (Parakeet, _) => (3.0, 0.0, 0.5),
                (Puffin, _) => (5.0, 3.0, 0.5),
                (Toucan, _) => (5.0, 3.0, 0.5),
                (BloodmoonBat, _) => (11.0, 6.0, 0.0),
                (VampireBat, _) => (5.0, 5.0, 0.0),
            },
            leg: match (body.species, body.body_type) {
                (SnowyOwl, _) => (1.5, -2.5, 4.8),
                (HornedOwl, _) => (1.5, -2.5, 4.8),
                (Duck, _) => (1.5, 0.0, 3.0),
                (Cockatiel, _) => (1.0, -1.0, 3.0),
                (Chicken, Male) => (1.0, 0.0, 4.4),
                (Chicken, Female) => (1.0, 0.0, 4.4),
                (Bat, _) => (2.5, -1.0, 6.0),
                (Penguin, _) => (1.5, -1.5, 4.5),
                (Goose, _) => (2.0, -2.5, 4.4),
                (Peacock, _) => (2.0, -2.0, 7.0),
                (Eagle, _) => (1.5, -4.0, 4.4),
                (Parrot, _) => (1.5, -1.0, 2.2),
                (Crow, _) => (1.5, -2.5, 2.1),
                (Dodo, _) => (1.5, -3.0, 3.0),
                (Parakeet, _) => (1.0, -2.0, 1.3),
                (Puffin, _) => (1.5, -2.2, 2.5),
                (Toucan, _) => (1.5, -3.0, 2.3),
                (BloodmoonBat, _) => (1.5, -3.5, 6.0),
                (VampireBat, _) => (2.5, -1.0, 6.0),
            },
            scaler: match (body.species, body.body_type) {
                (SnowyOwl, _) => 0.75,
                (HornedOwl, _) => 0.75,
                (Duck, _) => 0.75,
                (Cockatiel, _) => 0.75,
                (Chicken, _) => 0.75,
                (Bat, _) => 0.75,
                (Penguin, _) => 0.75,
                (Goose, _) => 0.75,
                (Peacock, _) => 0.75,
                (Eagle, _) => 0.75,
                (Parrot, _) => 0.75,
                (Crow, _) => 0.75,
                (Dodo, _) => 0.75,
                (Parakeet, _) => 0.75,
                (Puffin, _) => 0.75,
                (Toucan, _) => 0.75,
                (BloodmoonBat, _) => 1.05,
                (VampireBat, _) => 0.75,
            },
            feed: match (body.species, body.body_type) {
                (SnowyOwl, _) => -0.65,
                (HornedOwl, _) => -0.65,
                (Duck, _) => -0.55,
                (Cockatiel, _) => -0.60,
                (Chicken, _) => -0.65,
                (Bat, _) => -0.55,
                (Penguin, _) => -0.75,
                (Goose, _) => -0.65,
                (Peacock, _) => -1.2,
                (Eagle, _) => -0.75,
                (Parrot, _) => -0.60,
                (Crow, _) => -0.65,
                (Dodo, _) => -0.65,
                (Parakeet, _) => -0.60,
                (Puffin, _) => -0.75,
                (Toucan, _) => -0.50,
                (BloodmoonBat, _) => -0.55,
                (VampireBat, _) => -0.55,
            },
        }
    }
}

pub fn viewpoint(body: &Body) -> Vec4<f32> {
    use comp::bird_medium::Species::*;
    match body.species {
        Bat | BloodmoonBat | VampireBat => Vec4::new(0.0, 5.0, -4.0, 1.0),
        _ => Vec4::new(0.0, 3.0, 2.0, 1.0),
    }
}

pub fn mount_mat(
    computed_skeleton: &ComputedBirdMediumSkeleton,
    skeleton: &BirdMediumSkeleton,
) -> (Mat4<f32>, Quaternion<f32>) {
    (computed_skeleton.chest, skeleton.chest.orientation)
}

pub fn mount_transform(
    body: &Body,
    computed_skeleton: &ComputedBirdMediumSkeleton,
    skeleton: &BirdMediumSkeleton,
) -> Transform<f32, f32, f32> {
    use comp::bird_medium::{BodyType::*, Species::*};

    let mount_point = match (body.species, body.body_type) {
        (SnowyOwl, _) => (0.0, -4.0, 2.0),
        (HornedOwl, _) => (0.0, -4.0, 1.0),
        (Duck, _) => (0.0, -3.0, 2.0),
        (Cockatiel, _) => (0.0, -1.5, 1.5),
        (Chicken, Female) => (0.0, -2.5, 2.5),
        (Chicken, Male) => (0.0, -2.0, 2.5),
        (Bat, _) => (0.0, 0.0, -1.5),
        (Penguin, _) => (0.0, -2.5, 4.5),
        (Goose, _) => (0.0, -1.0, 3.0),
        (Peacock, _) => (0.0, -3.0, 2.5),
        (Eagle, _) => (0.0, -4.0, 2.0),
        (Parrot, _) => (0.0, -3.0, 1.5),
        (Crow, _) => (0.0, -1.5, 2.0),
        (Dodo, _) => (0.0, -4.0, 3.0),
        (Parakeet, _) => (0.0, -1.5, 1.5),
        (Puffin, _) => (0.0, -3.5, 1.0),
        (Toucan, _) => (0.0, -2.0, 1.5),
        (BloodmoonBat, _) => (0.0, 0.5, 3.5),
        (VampireBat, _) => (0.0, 0.0, -1.5),
    }
    .into();

    let (mount_mat, orientation) = mount_mat(computed_skeleton, skeleton);
    Transform {
        position: mount_mat.mul_point(mount_point),
        orientation,
        scale: Vec3::one(),
    }
}
