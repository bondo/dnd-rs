use bevy::{
    input::common_conditions::input_just_pressed, prelude::*, render::camera::ScalingMode,
    window::PrimaryWindow,
};

use dnd_rs_level::{CellFloor, CellKind, Level};

const UNIT_SIZE: f32 = 100.0;
const OFFSET: f32 = UNIT_SIZE / 2.0;

const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const TEXT_SIZE: f32 = UNIT_SIZE * 0.75;

const PADDING_TOP: f32 = 0.0;
const PADDING_LEFT: f32 = 0.0;
const PADDING_BOTTOM: f32 = UNIT_SIZE * 0.3;
const PADDING_RIGHT: f32 = UNIT_SIZE * 0.3;

const WALL_COLOR: Color = Color::srgb(0.0, 0.0, 1.0);
const MONSTER_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const TREASURE_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);
const CELL_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
const BORDER_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);

const BORDER_WIDTH: f32 = UNIT_SIZE * 0.05;
const CELL_SIZE: Vec2 = Vec2::new(UNIT_SIZE - BORDER_WIDTH, UNIT_SIZE - BORDER_WIDTH);

// TODO:
// - Spawn question mark on right click
// - Add indicator when row/column wall count matches
// - Add indicator when row/column has too many walls
// - Add indicator when monster is in a blind alley
// - Handle level completed
// - Add interface settings to change level size
// - Add Android support
// - Add Web support

pub struct DungeonsAndDiagramsPlugin;

impl Plugin for DungeonsAndDiagramsPlugin {
    fn build(&self, app: &mut App) {
        let level = Level::random(7, 7);
        // println!("{:?}", level);

        app.insert_resource(level)
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                handle_click.run_if(input_just_pressed(MouseButton::Left)),
            );
    }
}

#[derive(Component)]
struct MainCamera;

#[derive(Component, Default)]
struct Cell;

#[derive(Component, Default)]
struct Floor;

#[derive(Component, Default)]
struct Wall;

#[derive(Component, Default)]
struct Treasure;

#[derive(Component, Default)]
struct Monster;

#[derive(Bundle, Default)]
struct FloorBundle {
    cell: Cell,
    floor: Floor,
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
    let width = (level.width() as f32 + 1.0) * UNIT_SIZE + PADDING_LEFT + PADDING_RIGHT;
    let height = (level.height() as f32 + 1.0) * UNIT_SIZE + PADDING_TOP + PADDING_BOTTOM;

    let mut camera_2d = Camera2dBundle::default();
    camera_2d.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: width,
        min_height: height,
    };
    camera_2d.transform = Transform::from_xyz(width / 2.0, height / 2.0, 0.0);
    commands.spawn((camera_2d, MainCamera));

    // Paint border as background
    commands.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(
                (width - PADDING_LEFT - PADDING_RIGHT) / 2.0 + OFFSET + PADDING_LEFT,
                (height - PADDING_TOP - PADDING_BOTTOM) / 2.0 - OFFSET + PADDING_BOTTOM,
                -1.0,
            ),
            scale: Vec3::new(
                level.width() as f32 * UNIT_SIZE + BORDER_WIDTH,
                level.height() as f32 * UNIT_SIZE + BORDER_WIDTH,
                0.0,
            ),
            ..Default::default()
        },
        sprite: Sprite {
            color: BORDER_COLOR,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut column_headers = vec![0; level.width()];
    let mut row_headers = vec![0; level.height()];
    level.iter().for_each(|c| {
        if c.has_wall() {
            column_headers[c.x()] += 1;
            row_headers[c.y()] += 1;
        }

        let transform = Transform {
            translation: Vec3::new(
                (c.x() as f32 + 1.0) * UNIT_SIZE + OFFSET + PADDING_LEFT,
                height - ((c.y() as f32 + 1.0) * UNIT_SIZE + OFFSET + PADDING_TOP),
                0.0,
            ),
            scale: CELL_SIZE.extend(1.0),
            ..Default::default()
        };
        match c.kind() {
            CellKind::Wall | CellKind::Floor(CellFloor::Empty) => {
                commands.spawn((
                    FloorBundle {
                        ..Default::default()
                    },
                    SpriteBundle {
                        transform,
                        sprite: Sprite {
                            color: CELL_COLOR,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ));
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
                (i as f32 + 1.0) * UNIT_SIZE + OFFSET + PADDING_LEFT,
                height - (OFFSET + PADDING_TOP),
                100.0,
            ),
            ..Default::default()
        });
    });

    row_headers.iter().enumerate().for_each(|(i, value)| {
        commands.spawn(Text2dBundle {
            text: Text::from_section(value.to_string(), text_style.clone()),
            transform: Transform::from_xyz(
                OFFSET + PADDING_LEFT,
                height - ((i as f32 + 1.0) * UNIT_SIZE + OFFSET + PADDING_TOP),
                100.0,
            ),
            ..Default::default()
        });
    });
}

fn handle_click(
    mut commands: Commands,
    q_walls: Query<(Entity, &Transform), With<Wall>>,
    q_empty_cells: Query<(&Floor, &Transform), Without<Wall>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_windows.single();

    // Get cursor position in world coordinates
    let Some(cursor_position) = window.cursor_position() else {
        // Cursor is not in primary window
        return;
    };

    // Translate cursor position to world coordinates
    let Some(cursor_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    else {
        // Cursor is not in camera view
        return;
    };

    // If a wall is clicked, remove it
    for (entity, transform) in q_walls.iter() {
        if is_cursor_in_cell(cursor_position, transform) {
            commands.entity(entity).despawn();
            return;
        }
    }

    // If an empty cell is clicked, add a wall
    for (_floor, transform) in q_empty_cells.iter() {
        if is_cursor_in_cell(cursor_position, transform) {
            commands.spawn((
                Wall,
                SpriteBundle {
                    transform: Transform {
                        translation: transform.translation.with_z(1.0),
                        scale: CELL_SIZE.extend(1.0),
                        ..Default::default()
                    },
                    sprite: Sprite {
                        color: WALL_COLOR,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ));
            return;
        }
    }
}

fn is_cursor_in_cell(cursor: Vec2, cell: &Transform) -> bool {
    let center = cell.translation.xy();
    let half_size = CELL_SIZE / 2.0;

    let bottom_left = center - half_size;
    let top_right = center + half_size;

    cursor.x >= bottom_left.x
        && cursor.x <= top_right.x
        && cursor.y >= bottom_left.y
        && cursor.y <= top_right.y
}
