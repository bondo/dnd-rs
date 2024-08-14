use bevy::{prelude::*, render::camera::ScalingMode};

use dnd_rs_level::{CellFloor, CellKind, Level};

const UNIT_SIZE: f32 = 100.0;

const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const TEXT_SIZE: f32 = UNIT_SIZE;

const WALL_COLOR: Color = Color::srgb(0.0, 0.0, 1.0);
const MONSTER_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const TREASURE_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);
const FLOOR_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
// const BORDER_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);

const CELL_SIZE: Vec2 = Vec2::new(UNIT_SIZE, UNIT_SIZE);

pub struct DungeonsAndDiagramsPlugin;

impl Plugin for DungeonsAndDiagramsPlugin {
    fn build(&self, app: &mut App) {
        let level = Level::random(9, 9);
        println!("{:?}", level);

        app.insert_resource(level).add_systems(Startup, setup);
    }
}

#[derive(Component, Default)]
struct Cell;

#[derive(Component, Default)]
struct Wall;

#[derive(Component, Default)]
struct Treasure;

#[derive(Component, Default)]
struct Monster;

#[derive(Bundle, Default)]
struct FloorBundle {
    cell: Cell,
}

#[derive(Bundle, Default)]
struct WallBundle {
    cell: Cell,
    wall: Wall,
}

#[derive(Bundle, Default)]
struct MonsterBundle {
    cell: Cell,
    monster: Monster,
}

#[derive(Bundle, Default)]
struct TreasureBundle {
    cell: Cell,
    treasure: Treasure,
}

fn setup(mut commands: Commands, level: Res<Level>) {
    let padding_left = 2.0;
    let padding_right = 1.0;
    let padding_top = 2.0;
    let padding_bottom = 1.0;

    let width = (level.width() as f32 + padding_left + padding_right) * UNIT_SIZE;
    let height = (level.height() as f32 + padding_top * padding_bottom) * UNIT_SIZE;

    let mut camera_2d = Camera2dBundle::default();
    camera_2d.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: width,
        min_height: height,
    };
    camera_2d.transform = Transform::from_xyz(width / 2.0, height / 2.0, 0.0);
    commands.spawn(camera_2d);

    // Paint floor as background
    commands.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(width / 2.0, (height - UNIT_SIZE) / 2.0, -1.0),
            scale: Vec3::new(
                level.width() as f32 * UNIT_SIZE,
                level.height() as f32 * UNIT_SIZE,
                0.0,
            ),
            ..Default::default()
        },
        sprite: Sprite {
            color: FLOOR_COLOR,
            ..Default::default()
        },
        ..Default::default()
    });

    // TODO: Paint border on tiles at z = 1.0

    let mut column_headers = vec![0; level.width()];
    let mut row_headers = vec![0; level.height()];
    level.iter().for_each(|c| {
        if c.has_wall() {
            column_headers[c.x()] += 1;
            row_headers[c.y()] += 1;
        }

        let transform = Transform {
            translation: Vec3::new(
                (c.x() as f32 + padding_left) * UNIT_SIZE,
                height - (c.y() as f32 + padding_top) * UNIT_SIZE,
                0.0,
            ),
            scale: CELL_SIZE.extend(1.0),
            ..Default::default()
        };
        match c.kind() {
            CellKind::Wall => {
                commands.spawn((
                    WallBundle {
                        ..Default::default()
                    },
                    SpriteBundle {
                        transform,
                        sprite: Sprite {
                            color: WALL_COLOR,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ));
            }
            CellKind::Floor(CellFloor::Empty) => {
                commands.spawn(FloorBundle {
                    ..Default::default()
                });
            }
            CellKind::Floor(CellFloor::Treasure) => {
                commands.spawn((
                    TreasureBundle {
                        ..Default::default()
                    },
                    SpriteBundle {
                        transform,
                        sprite: Sprite {
                            color: TREASURE_COLOR,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ));
            }
            CellKind::Floor(CellFloor::Monster) => {
                commands.spawn((
                    MonsterBundle {
                        ..Default::default()
                    },
                    SpriteBundle {
                        transform,
                        sprite: Sprite {
                            color: MONSTER_COLOR,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ));
            }
        }
    });

    let text_style = TextStyle {
        font_size: TEXT_SIZE,
        color: TEXT_COLOR,
        ..Default::default()
    };

    column_headers.iter().enumerate().for_each(|(i, value)| {
        commands.spawn(Text2dBundle {
            text: Text::from_section(value.to_string(), text_style.clone()),
            transform: Transform::from_xyz(
                (i as f32 + padding_left) * UNIT_SIZE,
                height - UNIT_SIZE,
                100.0,
            ),
            ..Default::default()
        });
    });

    row_headers.iter().enumerate().for_each(|(i, value)| {
        commands.spawn(Text2dBundle {
            text: Text::from_section(value.to_string(), text_style.clone()),
            transform: Transform::from_xyz(
                UNIT_SIZE,
                height - (i as f32 + padding_top) * UNIT_SIZE,
                100.0,
            ),
            ..Default::default()
        });
    });
}
