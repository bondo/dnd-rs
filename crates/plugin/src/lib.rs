use std::cmp::Ordering;

use bevy::{
    input::common_conditions::input_just_pressed, prelude::*, render::camera::ScalingMode,
    window::PrimaryWindow,
};

use dnd_rs_level::{CellFloor, CellKind, Level};

const UNIT_SIZE: f32 = 100.0;
const OFFSET: f32 = UNIT_SIZE / 2.0;

const HEADER_TEXT_COLOR_TOO_FEW: Color = Color::srgb(0.5, 0.5, 1.0);
const HEADER_TEXT_COLOR_MATCH: Color = Color::srgb(0.0, 1.0, 0.0);
const HEADER_TEXT_COLOR_TOO_MANY: Color = Color::srgb(1.0, 0.0, 0.0);
const QUESTION_MARK_COLOR: Color = Color::srgb(1.0, 0.5, 0.0);

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
// - Handle long press events like right click
// - Add indicator when monster is in a blind alley
// - Handle level completed
// - Add interface settings to change level size
// - Add Android support
// - Add solver
// - Add property based testing (unique solution, proper layout)

pub struct DungeonsAndDiagramsPlugin;

impl Plugin for DungeonsAndDiagramsPlugin {
    fn build(&self, app: &mut App) {
        let level = Level::random(8, 8);
        // println!("{:?}", level);

        app.insert_resource(level)
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    handle_left_click.run_if(input_just_pressed(MouseButton::Left)),
                    handle_right_click.run_if(input_just_pressed(MouseButton::Right)),
                    handle_touch,
                    update_row_header_colors,
                    update_column_header_colors,
                ),
            );
    }
}

#[derive(Component)]
struct MainCamera;

#[derive(Component, Default)]
struct Cell;

#[derive(Component)]
struct HeaderText(usize);

#[derive(Component, Clone, Copy)]
struct Row(usize);

#[derive(Component, Clone, Copy)]
struct Column(usize);

#[derive(Component, Default)]
struct Floor;

#[derive(Component, Default)]
struct Wall;

#[derive(Component, Default)]
struct Treasure;

#[derive(Component, Default)]
struct Monster;

#[derive(Component, Default)]
struct QuestionMark;

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
        let pos = (Row(c.y()), Column(c.x()));
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
                    pos,
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
                    pos,
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
                    pos,
                ));
            }
        }
    });

    let text_style = TextStyle {
        font_size: TEXT_SIZE,
        color: HEADER_TEXT_COLOR_TOO_FEW,
        ..Default::default()
    };

    column_headers.iter().enumerate().for_each(|(i, value)| {
        commands.spawn((
            Text2dBundle {
                text: Text::from_section(value.to_string(), text_style.clone()),
                transform: Transform::from_xyz(
                    (i as f32 + 1.0) * UNIT_SIZE + OFFSET + PADDING_LEFT,
                    height - (OFFSET + PADDING_TOP),
                    100.0,
                ),
                ..Default::default()
            },
            Column(i),
            HeaderText(*value),
        ));
    });

    row_headers.iter().enumerate().for_each(|(i, value)| {
        commands.spawn((
            Text2dBundle {
                text: Text::from_section(value.to_string(), text_style.clone()),
                transform: Transform::from_xyz(
                    OFFSET + PADDING_LEFT,
                    height - ((i as f32 + 1.0) * UNIT_SIZE + OFFSET + PADDING_TOP),
                    100.0,
                ),
                ..Default::default()
            },
            Row(i),
            HeaderText(*value),
        ));
    });
}

fn handle_left_click(
    mut commands: Commands,
    q_walls: Query<(Entity, &Transform), With<Wall>>,
    q_empty_cells: Query<(&Floor, &Transform, &Row, &Column), Without<Wall>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let Some(cursor_position) = get_cursor_position_in_world(q_windows, q_camera) else {
        // Cursor is not in camera view
        return;
    };

    execute_primary_action(&mut commands, &q_walls, &q_empty_cells, cursor_position);
}

fn handle_touch(
    mut touch_events: EventReader<TouchInput>,
    mut commands: Commands,
    q_walls: Query<(Entity, &Transform), With<Wall>>,
    q_empty_cells: Query<(&Floor, &Transform, &Row, &Column), Without<Wall>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    use bevy::input::touch::TouchPhase;
    for event in touch_events.read() {
        if event.phase == TouchPhase::Ended {
            let Some(position) = viewport_to_world_position(&q_camera, event.position) else {
                continue;
            };

            execute_primary_action(&mut commands, &q_walls, &q_empty_cells, position);
        }
    }
}

fn execute_primary_action(
    commands: &mut Commands,
    q_walls: &Query<(Entity, &Transform), With<Wall>>,
    q_empty_cells: &Query<(&Floor, &Transform, &Row, &Column), Without<Wall>>,
    pos: Vec2,
) {
    // If a wall is clicked, remove it
    for (entity, transform) in q_walls.iter() {
        if is_cursor_in_cell(pos, transform) {
            commands.entity(entity).despawn();
            return;
        }
    }

    // If an empty cell is clicked, add a wall
    for (_floor, transform, row, column) in q_empty_cells.iter() {
        if is_cursor_in_cell(pos, transform) {
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
                *row,
                *column,
            ));
            return;
        }
    }
}

fn handle_right_click(
    mut commands: Commands,
    q_question_marks: Query<(Entity, &Transform), With<QuestionMark>>,
    q_cells: Query<(&Cell, &Transform, &Row, &Column), Without<QuestionMark>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let Some(cursor_position) = get_cursor_position_in_world(q_windows, q_camera) else {
        // Cursor is not in camera view
        return;
    };

    // If a question mark is clicked, remove it
    for (entity, transform) in q_question_marks.iter() {
        if is_cursor_in_cell(cursor_position, transform) {
            commands.entity(entity).despawn();
            return;
        }
    }

    // If a cell is clicked, add a question mark
    for (_cell, transform, row, column) in q_cells.iter() {
        if is_cursor_in_cell(cursor_position, transform) {
            commands.spawn((
                QuestionMark,
                Text2dBundle {
                    text: Text::from_section(
                        "?",
                        TextStyle {
                            font_size: TEXT_SIZE,
                            color: QUESTION_MARK_COLOR,
                            ..Default::default()
                        },
                    ),
                    transform: Transform {
                        translation: transform.translation.with_z(2.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                *row,
                *column,
            ));
            return;
        }
    }
}

fn update_row_header_colors(
    mut q_row_headers: Query<(&Row, &mut Text, &HeaderText)>,
    q_walls: Query<&Row, With<Wall>>,
    level: Res<Level>,
) {
    // Count walls in each row and column
    let mut row_walls = vec![0; level.height()];
    q_walls.iter().for_each(|row| {
        row_walls[row.0] += 1;
    });

    // Update row headers colors
    q_row_headers
        .iter_mut()
        .for_each(|(row, mut text, header_text)| {
            let row_wall_count = row_walls[row.0];
            text.sections[0].style.color = match row_wall_count.cmp(&header_text.0) {
                Ordering::Less => HEADER_TEXT_COLOR_TOO_FEW,
                Ordering::Equal => HEADER_TEXT_COLOR_MATCH,
                Ordering::Greater => HEADER_TEXT_COLOR_TOO_MANY,
            };
        });
}

fn update_column_header_colors(
    mut q_column_headers: Query<(&Column, &mut Text, &HeaderText)>,
    q_walls: Query<&Column, With<Wall>>,
    level: Res<Level>,
) {
    // Count walls in each row and column
    let mut column_walls = vec![0; level.width()];
    q_walls.iter().for_each(|column| {
        column_walls[column.0] += 1;
    });

    // Update column headers colors
    q_column_headers
        .iter_mut()
        .for_each(|(column, mut text, header_text)| {
            let column_wall_count = column_walls[column.0];
            text.sections[0].style.color = match column_wall_count.cmp(&header_text.0) {
                Ordering::Less => HEADER_TEXT_COLOR_TOO_FEW,
                Ordering::Equal => HEADER_TEXT_COLOR_MATCH,
                Ordering::Greater => HEADER_TEXT_COLOR_TOO_MANY,
            };
        });
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

fn get_cursor_position_in_world(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Option<Vec2> {
    viewport_to_world_position(&q_camera, q_windows.single().cursor_position()?)
}

fn viewport_to_world_position(
    q_camera: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    position: Vec2,
) -> Option<Vec2> {
    let (camera, camera_transform) = q_camera.single();
    camera.viewport_to_world_2d(camera_transform, position)
}
