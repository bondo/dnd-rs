use bevy::{prelude::*, render::camera::ScalingMode};

use dnd_rs_level::{CellFloor, CellKind, Level};

const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const TEXT_SPACING: f32 = 60.0;
const TEXT_OFFSET: f32 = 30.0;
const TEXT_SIZE: f32 = 50.0;

const WALL_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
const MONSTER_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const TREASURE_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);

const CELL_SIZE: Vec2 = Vec2::new(1.0, 1.0);

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
    let padding_left = 1.5;
    let padding_top = 2.0;

    let width = level.width() as f32 + padding_left * 2.0;
    let height = level.height() as f32 + padding_top * 2.0;

    let mut camera_2d = Camera2dBundle::default();
    camera_2d.projection.scaling_mode = ScalingMode::Fixed { width, height };
    camera_2d.transform = Transform::from_xyz(width / 2.0, height / 2.0, 0.0);
    commands.spawn(camera_2d);

    let mut column_headers = vec![0; level.width()];
    let mut row_headers = vec![0; level.height()];
    level.iter().for_each(|c| {
        if c.has_wall() {
            column_headers[c.x()] += 1;
            row_headers[c.y()] += 1;
        }

        let transform = Transform {
            translation: Vec3::new(
                c.x() as f32 + padding_left,
                height - c.y() as f32 - padding_top,
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

    column_headers.iter().enumerate().for_each(|(i, value)| {
        commands.spawn(
            TextBundle::from_sections([TextSection::new(
                value.to_string(),
                TextStyle {
                    font_size: TEXT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            )])
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(TEXT_OFFSET),
                left: Val::Px(TEXT_OFFSET + (i as f32 + 1.0) * TEXT_SPACING),
                ..default()
            }),
        );
    });

    row_headers.iter().enumerate().for_each(|(i, value)| {
        commands.spawn(
            TextBundle::from_sections([TextSection::new(
                value.to_string(),
                TextStyle {
                    font_size: TEXT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            )])
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(TEXT_OFFSET + (i as f32 + 1.0) * TEXT_SPACING),
                left: Val::Px(TEXT_OFFSET),
                ..default()
            }),
        );
    });
}

// fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
//     // update our timer with the time elapsed since the last update
//     // if that caused the timer to finish, we say hello to everyone
//     if timer.0.tick(time.delta()).just_finished() {
//         for name in &query {
//             println!("hello {}!", name.0);
//         }
//     }
// }

// fn update_people(mut query: Query<&mut Name, With<Person>>) {
//     for mut name in &mut query {
//         if name.0 == "Elaina Proctor" {
//             name.0 = "Elaina Hume".to_string();
//             break; // We don’t need to change any other names
//         }
//     }
// }
