use super::*;
use crate::{
    site2::util::gradient::WrapMode,
    util::{RandomField, Sampler, DIAGONALS, LOCALITY, NEIGHBORS},
    Land,
};
use common::{
    comp::Content,
    generation::SpecialEntity,
    terrain::{BlockKind, SpriteCfg, SpriteKind},
};
use rand::prelude::*;
use std::{f32::consts::TAU, mem};
use vek::*;

/// Represents house data generated by the `generate()` method
pub struct CliffTownAirshipDock {
    /// Tile position of the door tile
    pub door_tile: Vec2<i32>,
    /// Approximate altitude of the door tile
    pub(crate) alt: i32,
    door_dir: Vec2<i32>,
    surface_color: Rgb<f32>,
    sub_surface_color: Rgb<f32>,
    center: Vec2<i32>,
    variant: i32,
    storeys: i32,
    platform_length: i32,
    pub docking_positions: Vec<Vec3<i32>>,
}

impl CliffTownAirshipDock {
    pub fn generate(
        land: &Land,
        index: IndexRef,
        _rng: &mut impl Rng,
        site: &Site,
        door_tile: Vec2<i32>,
        door_dir: Vec2<i32>,
        tile_aabr: Aabr<i32>,
    ) -> Self {
        let door_tile_pos = site.tile_center_wpos(door_tile);
        let bounds = Aabr {
            min: site.tile_wpos(tile_aabr.min),
            max: site.tile_wpos(tile_aabr.max),
        };
        let center = bounds.center();
        let alt = land.get_alt_approx(door_tile_pos) as i32;
        let variant = 15;
        let storeys = 5 + (variant / 2);
        let platform_length = 2 * variant;
        let mut docking_positions = vec![];
        let mut platform_level = alt - 40;
        let mut platform_height = 18 + variant / 2;
        for s in 0..storeys {
            if s == (storeys - 1) {
                for dir in CARDINALS {
                    let docking_pos = center + dir * (platform_length + 7);
                    docking_positions.push(docking_pos.with_z(platform_level + 1));
                }
            }
            platform_height += -1;
            platform_level += platform_height;
        }

        let (surface_color, sub_surface_color) =
            if let Some(sample) = land.column_sample(bounds.center(), index) {
                (sample.surface_color, sample.sub_surface_color)
            } else {
                (Rgb::new(161.0, 116.0, 86.0), Rgb::new(88.0, 64.0, 64.0))
            };
        Self {
            door_tile: door_tile_pos,
            alt,
            door_dir,
            surface_color,
            sub_surface_color,
            center,
            variant,
            storeys,
            platform_length,
            docking_positions,
        }
    }
}

impl Structure for CliffTownAirshipDock {
    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"render_cliff_town_airship_dock\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "render_cliff_town_airship_dock")]
    fn render_inner(&self, _site: &Site, _land: &Land, painter: &Painter) {
        let base = self.alt;
        let plot_center = self.center;
        let door_dir = self.door_dir;

        let surface_color = self.surface_color.map(|e| (e * 255.0) as u8);
        let sub_surface_color = self.sub_surface_color.map(|e| (e * 255.0) as u8);
        let gradient_center = Vec3::new(
            plot_center.x as f32,
            plot_center.y as f32,
            (base + 1) as f32,
        );
        let gradient_var_1 = RandomField::new(0).get(plot_center.with_z(base)) as i32 % 8;
        let gradient_var_2 = RandomField::new(0).get(plot_center.with_z(base + 1)) as i32 % 10;

        let brick = Fill::Gradient(
            util::gradient::Gradient::new(
                gradient_center,
                8.0 + gradient_var_1 as f32,
                util::gradient::Shape::Point,
                (surface_color, sub_surface_color),
            )
            .with_repeat(if gradient_var_2 > 5 {
                WrapMode::Repeat
            } else {
                WrapMode::PingPong
            }),
            BlockKind::Rock,
        );

        let wood = Fill::Brick(BlockKind::Wood, Rgb::new(106, 83, 51), 12);
        let color = Fill::Block(Block::air(SpriteKind::CliffDecorBlock));
        let window = Fill::Block(Block::air(SpriteKind::WindowArabic));
        let window2 = Fill::Block(Block::air(SpriteKind::WindowArabic).with_ori(2).unwrap());
        let rope = Fill::Block(Block::air(SpriteKind::Rope));

        let tube_var = RandomField::new(0).get(plot_center.with_z(base)) as i32 % 6;
        let radius = 10.0 + tube_var as f32;
        let tubes = 3.0 + tube_var as f32;
        let phi = TAU / tubes;
        for n in 1..=tubes as i32 {
            let center = Vec2::new(
                plot_center.x + (radius * ((n as f32 * phi).cos())) as i32,
                plot_center.y + (radius * ((n as f32 * phi).sin())) as i32,
            );
            // common superquadric degree for rooms
            let sq_type = 3.5;
            let storeys = self.storeys;
            let variant = self.variant;
            let mut length = 16 + (variant / 2);
            let mut width = 7 * length / 8;
            let mut height = 18 + variant / 2;
            let mut floor_level = self.alt - 40;
            let platform_length = self.platform_length;
            let mut ground_entries = 0;
            for s in 0..storeys {
                let x_offset = RandomField::new(0).get((center - length).with_z(base)) as i32 % 10;
                let y_offset = RandomField::new(0).get((center + length).with_z(base)) as i32 % 10;
                let super_center =
                    Vec2::new(center.x - 3 + x_offset / 2, center.y - 3 + y_offset / 2);
                // CliffTower Hoodoo Overlay
                painter
                    .cubic_bezier(
                        super_center.with_z(floor_level + (height / 2)),
                        (super_center - x_offset).with_z(floor_level + height),
                        (super_center - y_offset).with_z(floor_level + (height) + (height / 2)),
                        super_center.with_z(floor_level + (2 * height)),
                        (length - 1) as f32,
                    )
                    .fill(brick.clone());
                if s == (storeys - 1) {
                    for dir in LOCALITY {
                        let cone_pos = super_center + (dir * 2);
                        let cone_var =
                            4 + RandomField::new(0).get(cone_pos.with_z(base)) as i32 % 4;
                        painter
                            .cone_with_radius(
                                cone_pos.with_z(floor_level + (2 * height) + 5),
                                (length / 2) as f32,
                                (length + cone_var) as f32,
                            )
                            .fill(brick.clone());
                    }
                }
                // center tube with  rooms
                if n == tubes as i32 {
                    // ground_entries
                    if ground_entries < 1 && floor_level > (base - 6) {
                        for dir in CARDINALS {
                            let entry_pos_inner = plot_center + (dir * (2 * length) - 4);
                            let entry_pos_outer = plot_center + (dir * (3 * length) + 4);
                            painter
                                .line(
                                    entry_pos_inner.with_z(floor_level + 6),
                                    entry_pos_outer.with_z(base + 35),
                                    6.0,
                                )
                                .clear();
                        }
                        let door_start = plot_center + door_dir * ((3 * (length / 2)) + 1);
                        painter
                            .line(
                                door_start.with_z(floor_level + 2),
                                self.door_tile.with_z(base),
                                4.0,
                            )
                            .fill(wood.clone());
                        painter
                            .line(
                                door_start.with_z(floor_level + 7),
                                self.door_tile.with_z(base + 6),
                                7.0,
                            )
                            .clear();
                        ground_entries += 1;
                    }
                    painter
                        .cubic_bezier(
                            plot_center.with_z(floor_level + (height / 2)),
                            (plot_center - x_offset).with_z(floor_level + height),
                            (plot_center - y_offset).with_z(floor_level + (height) + (height / 2)),
                            plot_center.with_z(floor_level + (2 * height)),
                            (length + 2) as f32,
                        )
                        .fill(brick.clone());
                    // platform
                    if s == (storeys - 1) {
                        let limit_up = painter.aabb(Aabb {
                            min: (plot_center - platform_length - 2).with_z(floor_level - 4),
                            max: (plot_center + platform_length + 2).with_z(floor_level + 1),
                        });
                        painter
                            .superquadric(
                                Aabb {
                                    min: (plot_center - platform_length - 2)
                                        .with_z(floor_level - 4),
                                    max: (plot_center + platform_length + 2)
                                        .with_z(floor_level + 6),
                                },
                                4.0,
                            )
                            .intersect(limit_up)
                            .fill(wood.clone());
                        // lanterns & cargo
                        for dir in NEIGHBORS {
                            let lantern_pos = plot_center + (dir * (platform_length - 6));

                            painter.sprite(
                                lantern_pos.with_z(floor_level + 1),
                                SpriteKind::StreetLamp,
                            );
                        }
                        for dir in DIAGONALS {
                            let cargo_pos = plot_center + (dir * (2 * length));

                            for dir in CARDINALS {
                                let sprite_pos = cargo_pos + dir;
                                let rows =
                                    (RandomField::new(0).get(sprite_pos.with_z(base)) % 3) as i32;
                                for r in 0..rows {
                                    painter
                                        .aabb(Aabb {
                                            min: (sprite_pos).with_z(floor_level + 1 + r),
                                            max: (sprite_pos + 1).with_z(floor_level + 2 + r),
                                        })
                                        .fill(Fill::Block(Block::air(
                                            match (RandomField::new(0)
                                                .get(sprite_pos.with_z(base + r))
                                                % 2)
                                                as i32
                                            {
                                                0 => SpriteKind::Barrel,
                                                _ => SpriteKind::CrateBlock,
                                            },
                                        )));
                                    if r > 0 {
                                        painter.owned_resource_sprite(
                                            sprite_pos.with_z(floor_level + 2 + r),
                                            SpriteKind::Crate,
                                            0,
                                        );
                                    }
                                }
                            }
                        }
                        for dir in CARDINALS {
                            // docks
                            let dock_pos = plot_center + (dir * platform_length);

                            painter
                                .cylinder(Aabb {
                                    min: (dock_pos - 8).with_z(floor_level),
                                    max: (dock_pos + 8).with_z(floor_level + 1),
                                })
                                .fill(wood.clone());
                            painter
                                .cylinder(Aabb {
                                    min: (dock_pos - 7).with_z(floor_level - 1),
                                    max: (dock_pos + 7).with_z(floor_level),
                                })
                                .fill(wood.clone());
                        }
                        // campfire
                        let campfire_pos =
                            Vec2::new(plot_center.x - platform_length - 2, plot_center.y)
                                .with_z(floor_level);
                        painter.spawn(
                            EntityInfo::at(campfire_pos.map(|e| e as f32 + 0.5))
                                .into_special(SpecialEntity::Waypoint),
                        );
                    }

                    // clear rooms and entries & decor
                    if floor_level > (base - 6) {
                        // decor
                        painter
                            .line(
                                Vec2::new(plot_center.x, plot_center.y - length)
                                    .with_z(floor_level + 5),
                                Vec2::new(plot_center.x, plot_center.y + length)
                                    .with_z(floor_level + 5),
                                4.0,
                            )
                            .fill(color.clone());
                        painter
                            .line(
                                Vec2::new(plot_center.x - length, plot_center.y)
                                    .with_z(floor_level + 5),
                                Vec2::new(plot_center.x + length, plot_center.y)
                                    .with_z(floor_level + 5),
                                4.0,
                            )
                            .fill(color.clone());
                        // entries
                        painter
                            .line(
                                Vec2::new(plot_center.x, plot_center.y - (2 * length) - 4)
                                    .with_z(floor_level + 4),
                                Vec2::new(plot_center.x, plot_center.y + (2 * length) + 4)
                                    .with_z(floor_level + 4),
                                4.0,
                            )
                            .clear();
                        painter
                            .line(
                                Vec2::new(plot_center.x - (2 * length) - 4, plot_center.y)
                                    .with_z(floor_level + 4),
                                Vec2::new(plot_center.x + (2 * length) + 4, plot_center.y)
                                    .with_z(floor_level + 4),
                                4.0,
                            )
                            .clear();
                        painter
                            .superquadric(
                                Aabb {
                                    min: (plot_center - length - 1).with_z(floor_level),
                                    max: (plot_center + length + 1)
                                        .with_z(floor_level + height - 4),
                                },
                                sq_type,
                            )
                            .clear();
                        // room floor
                        painter
                            .cylinder(Aabb {
                                min: (plot_center - length - 3).with_z(floor_level),
                                max: (plot_center + length + 3).with_z(floor_level + 1),
                            })
                            .fill(brick.clone());
                        painter
                            .cylinder(Aabb {
                                min: (plot_center - length + 1).with_z(floor_level),
                                max: (plot_center + length - 1).with_z(floor_level + 1),
                            })
                            .fill(color.clone());
                        painter
                            .cylinder(Aabb {
                                min: (plot_center - length + 2).with_z(floor_level),
                                max: (plot_center + length - 2).with_z(floor_level + 1),
                            })
                            .fill(brick.clone());
                        // entry sprites
                        painter
                            .aabb(Aabb {
                                min: Vec2::new(plot_center.x - 3, plot_center.y + length)
                                    .with_z(floor_level + 2),
                                max: Vec2::new(plot_center.x + 4, plot_center.y + length + 1)
                                    .with_z(floor_level + 7),
                            })
                            .fill(window2.clone());
                        painter
                            .aabb(Aabb {
                                min: Vec2::new(plot_center.x - 2, plot_center.y + length)
                                    .with_z(floor_level + 2),
                                max: Vec2::new(plot_center.x + 3, plot_center.y + length + 1)
                                    .with_z(floor_level + 7),
                            })
                            .clear();

                        painter
                            .aabb(Aabb {
                                min: Vec2::new(plot_center.x - 3, plot_center.y - length - 1)
                                    .with_z(floor_level + 2),
                                max: Vec2::new(plot_center.x + 4, plot_center.y - length)
                                    .with_z(floor_level + 7),
                            })
                            .fill(window2.clone());
                        painter
                            .aabb(Aabb {
                                min: Vec2::new(plot_center.x - 2, plot_center.y - length - 1)
                                    .with_z(floor_level + 2),
                                max: Vec2::new(plot_center.x + 3, plot_center.y - length)
                                    .with_z(floor_level + 7),
                            })
                            .clear();
                        painter
                            .aabb(Aabb {
                                min: Vec2::new(plot_center.x + length, plot_center.y - 3)
                                    .with_z(floor_level + 2),
                                max: Vec2::new(plot_center.x + length + 1, plot_center.y + 4)
                                    .with_z(floor_level + 7),
                            })
                            .fill(window.clone());
                        painter
                            .aabb(Aabb {
                                min: Vec2::new(plot_center.x + length, plot_center.y - 2)
                                    .with_z(floor_level + 2),
                                max: Vec2::new(plot_center.x + length + 1, plot_center.y + 3)
                                    .with_z(floor_level + 7),
                            })
                            .clear();

                        painter
                            .aabb(Aabb {
                                min: Vec2::new(plot_center.x - length - 1, plot_center.y - 3)
                                    .with_z(floor_level + 2),
                                max: Vec2::new(plot_center.x - length, plot_center.y + 4)
                                    .with_z(floor_level + 7),
                            })
                            .fill(window.clone());
                        painter
                            .aabb(Aabb {
                                min: Vec2::new(plot_center.x - length - 1, plot_center.y - 2)
                                    .with_z(floor_level + 2),
                                max: Vec2::new(plot_center.x - length, plot_center.y + 3)
                                    .with_z(floor_level + 7),
                            })
                            .clear();
                        // cargo in rooms
                        for dir in DIAGONALS {
                            let cargo_pos = plot_center + (dir * (length / 2));
                            for dir in CARDINALS {
                                let sprite_pos = cargo_pos + dir;
                                let rows =
                                    (RandomField::new(0).get(sprite_pos.with_z(base)) % 4) as i32;
                                for r in 0..rows {
                                    painter
                                        .aabb(Aabb {
                                            min: (sprite_pos).with_z(floor_level + 1 + r),
                                            max: (sprite_pos + 1).with_z(floor_level + 2 + r),
                                        })
                                        .fill(Fill::Block(Block::air(
                                            match (RandomField::new(0)
                                                .get(sprite_pos.with_z(base + r))
                                                % 2)
                                                as i32
                                            {
                                                0 => SpriteKind::Barrel,
                                                _ => SpriteKind::CrateBlock,
                                            },
                                        )));
                                }
                            }
                        }

                        // wall lamps
                        let corner_pos_1 = Vec2::new(plot_center.x - length, plot_center.y - 5);
                        let corner_pos_2 = Vec2::new(plot_center.x - 5, plot_center.y - length);
                        for dir in SQUARE_4 {
                            let lamp_pos_1 = Vec2::new(
                                corner_pos_1.x + (dir.x * ((2 * length) - 1)),
                                corner_pos_1.y + (dir.y * 10),
                            )
                            .with_z(floor_level + 7);
                            painter.rotated_sprite(
                                lamp_pos_1,
                                SpriteKind::WallLampMesa,
                                (2 + (4 * dir.x)) as u8,
                            );
                            let lamp_pos_2 = Vec2::new(
                                corner_pos_2.x + (dir.x * 10),
                                corner_pos_2.y + (dir.y * ((2 * length) - 1)),
                            )
                            .with_z(floor_level + 7);
                            painter.rotated_sprite(
                                lamp_pos_2,
                                SpriteKind::WallLampMesa,
                                (4 - (4 * dir.y)) as u8,
                            );
                        }
                    }
                    // stairs
                    if floor_level > (base + 8) {
                        let stairs_level = floor_level + 1;
                        let stairs_start = plot_center + door_dir * ((2 * length) - 7);
                        let mid_dir = if door_dir.x != 0 {
                            door_dir.x
                        } else {
                            door_dir.y
                        };
                        let stairs_mid = Vec2::new(
                            plot_center.x + mid_dir * (3 * (length / 2)),
                            plot_center.y + mid_dir * (3 * (length / 2)),
                        );
                        let stairs_end = Vec2::new(
                            plot_center.x + door_dir.y * ((2 * length) - 7),
                            plot_center.y + door_dir.x * ((2 * length) - 7),
                        );
                        let rope_pos = Vec2::new(
                            plot_center.x + mid_dir * ((3 * (length / 2)) + 2),
                            plot_center.y + mid_dir * ((3 * (length / 2)) + 2),
                        );

                        painter
                            .cylinder(Aabb {
                                min: (stairs_start - 6).with_z(stairs_level - 1),
                                max: (stairs_start + 6).with_z(stairs_level),
                            })
                            .fill(wood.clone());

                        painter
                            .cylinder(Aabb {
                                min: (stairs_mid - 6).with_z(stairs_level - (height / 2) - 1),
                                max: (stairs_mid + 6).with_z(stairs_level - (height / 2)),
                            })
                            .fill(wood.clone());

                        painter
                            .cylinder(Aabb {
                                min: (stairs_end - 6).with_z(stairs_level - height - 1),
                                max: (stairs_end + 6).with_z(stairs_level - height),
                            })
                            .fill(wood.clone());

                        for n in 0..2 {
                            let stairs = painter
                                .line(
                                    stairs_start.with_z(stairs_level + (n * 2)),
                                    stairs_mid.with_z(stairs_level - (height / 2) + (n * 2)),
                                    4.0 + (n as f32 / 2.0),
                                )
                                .union(painter.line(
                                    stairs_mid.with_z(stairs_level - (height / 2) + (n * 2)),
                                    stairs_end.with_z(stairs_level - height + (n * 2)),
                                    4.0 + (n as f32 / 2.0),
                                ));
                            match n {
                                0 => stairs.fill(wood.clone()),
                                _ => stairs.clear(),
                            };
                        }
                        painter
                            .line(
                                rope_pos.with_z(stairs_level + (height / 2) - 3),
                                (plot_center - (length / 2))
                                    .with_z(stairs_level + (height / 2) + 2),
                                1.5,
                            )
                            .fill(wood.clone());

                        painter
                            .aabb(Aabb {
                                min: rope_pos.with_z(stairs_level - (height / 2) - 1),
                                max: (rope_pos + 1).with_z(stairs_level + (height / 2) - 3),
                            })
                            .fill(rope.clone());
                    }
                }
                // vary next storey
                length += -1;
                width += -1;
                height += -1;
                floor_level += height;
                mem::swap(&mut length, &mut width);
            }
        }
        for dock_pos in &self.docking_positions {
            painter.rotated_sprite_with_cfg(
                *dock_pos,
                SpriteKind::Sign,
                Dir::from_vec2(dock_pos.xy() - self.center).sprite_ori(),
                SpriteCfg {
                    unlock: None,
                    content: Some(Content::localized("common-signs-airship_dock")),
                },
            );
        }
    }
}