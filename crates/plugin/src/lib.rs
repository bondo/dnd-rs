use std::cmp::Ordering;

use bevy::{
    input::{common_conditions::input_just_pressed, touch::TouchPhase},
    prelude::*,
    render::camera::ScalingMode,
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
// - Add interface settings to change level size
// - Add Android support

pub struct DungeonsAndDiagramsPlugin;

impl Plugin for DungeonsAndDiagramsPlugin {
    fn build(&self, app: &mut App) {
        let config = Config {
            width: 8,
            height: 8,
        };

        app.init_state::<AppState>()
            .insert_resource(config)
            .insert_resource(RandomSource(fastrand::Rng::new()))
            .add_systems(
                OnEnter(AppState::Playing),
                (generate_level, spawn_game).chain(),
            )
            .add_systems(OnEnter(AppState::Won), spawn_confetti)
            .add_systems(OnExit(AppState::Won), despawn_game)
            .add_systems(
                Update,
                (
                    (
                        handle_left_click.run_if(input_just_pressed(MouseButton::Left)),
                        handle_right_click.run_if(input_just_pressed(MouseButton::Right)),
                        handle_touch,
                        update_row_header_colors,
                        update_column_header_colors,
                        check_level_completed,
                    )
                        .run_if(in_state(AppState::Playing)),
                    (
                        move_confetti,
                        handle_won_state_input,
                        update_watch_confetti_timer,
                    )
                        .run_if(in_state(AppState::Won)),
                ),
            );
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Playing,
    Won,
}

#[derive(Resource)]
struct Config {
    width: usize,
    height: usize,
}

#[derive(Resource, Deref, DerefMut)]
struct RandomSource(fastrand::Rng);

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct GameComponent;

#[derive(Component, Default)]
struct Cell;

#[derive(Component, Deref)]
struct HeaderText(usize);

#[derive(Component, Clone, Copy, Deref)]
struct Row(usize);

#[derive(Component, Clone, Copy, Deref)]
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

fn generate_level(mut commands: Commands, config: Res<Config>) {
    commands.spawn((
        GameComponent,
        Level::random_unique_solution(config.width, config.height),
    ));
}

fn spawn_game(mut commands: Commands, q_level: Query<&Level>) {
    let level = q_level.single();

    let width = (level.width() as f32 + 1.0) * UNIT_SIZE + PADDING_LEFT + PADDING_RIGHT;
    let height = (level.height() as f32 + 1.0) * UNIT_SIZE + PADDING_TOP + PADDING_BOTTOM;

    let mut camera_2d = Camera2dBundle::default();
    camera_2d.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: width,
        min_height: height,
    };
    camera_2d.transform = Transform::from_xyz(width / 2.0, height / 2.0, 0.0);
    commands.spawn((GameComponent, camera_2d, MainCamera));

    // Paint border as background
    commands.spawn((
        GameComponent,
        SpriteBundle {
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
        },
    ));

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
                    GameComponent,
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
                    GameComponent,
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
                    GameComponent,
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
            GameComponent,
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
            GameComponent,
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
                GameComponent,
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
                GameComponent,
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
    q_level: Query<&Level>,
) {
    let level = q_level.single();

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
    q_level: Query<&Level>,
) {
    let level = q_level.single();

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

#[derive(Component)]
struct Confetti {
    velocity: Vec3,
    stretch_velocity: f32,
    rotation_velocity: f32,
}

#[derive(Component, Deref, DerefMut)]
struct WatchConfettiTimer(Timer);

// Display confetti when level is completed
fn check_level_completed(
    mut next_state: ResMut<NextState<AppState>>,
    q_walls: Query<(&Row, &Column), With<Wall>>,
    q_level: Query<&Level>,
) {
    let level = q_level.single();

    let is_completed = q_walls.iter().count() == level.iter().filter(|c| c.has_wall()).count()
        && q_walls.iter().all(|(r, c)| level.is_wall(c.0, r.0));
    if is_completed {
        info!("Level completed!");
        next_state.set(AppState::Won);
    }
}

fn spawn_confetti(mut commands: Commands, mut rnd: ResMut<RandomSource>, q_level: Query<&Level>) {
    let level = q_level.single();

    let width = (level.width() as f32 + 1.0) * UNIT_SIZE + PADDING_LEFT + PADDING_RIGHT;

    for _ in 0..1000 {
        let x = rnd.f32() * width;
        let y = 0.0;

        let color = Color::srgb(rnd.f32(), rnd.f32(), rnd.f32());
        commands.spawn((
            GameComponent,
            Confetti {
                velocity: Vec3::new(rnd.f32() * 1000.0 - 500.0, rnd.f32() * 700.0, 0.0),
                stretch_velocity: rnd.f32() * 10.0 + 10.0,
                rotation_velocity: rnd.f32() * 10.0,
            },
            SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(x, y, 200.0),
                    scale: Vec3::new(rnd.f32() * 5.0 + 5.0, rnd.f32() * 5.0 + 5.0, 0.0),
                    ..Default::default()
                },
                sprite: Sprite {
                    color,
                    ..Default::default()
                },
                ..Default::default()
            },
        ));
    }

    commands.spawn((
        GameComponent,
        WatchConfettiTimer(Timer::from_seconds(1.0, TimerMode::Once)),
    ));

    // TODO: Spawn some text and button to restart
}

fn despawn_game(mut commands: Commands, q_entity: Query<Entity, With<GameComponent>>) {
    for entity in &q_entity {
        commands.entity(entity).despawn();
    }
}

fn move_confetti(time: Res<Time>, mut q_confetti: Query<(&mut Transform, &mut Confetti)>) {
    for (mut transform, mut confetti) in q_confetti.iter_mut() {
        transform.translation += confetti.velocity * time.delta_seconds();

        // Add gravity
        confetti.velocity.y -= 200.0 * time.delta_seconds();

        // Rotate
        transform.rotation *=
            Quat::from_rotation_z(confetti.rotation_velocity * time.delta_seconds());

        // Stretch
        transform.scale.x += confetti.stretch_velocity * time.delta_seconds();

        // Reverse stretch when reaching a limit
        if transform.scale.x > 20.0 || transform.scale.x < 5.0 {
            confetti.stretch_velocity = -confetti.stretch_velocity;
        }

        // Increase stretch and rotation velocity while falling
        if confetti.velocity.y < 0.0 {
            confetti.stretch_velocity += 10.0 * time.delta_seconds();
            confetti.rotation_velocity += 2.0 * time.delta_seconds();
        }
    }
}

fn update_watch_confetti_timer(mut q_timer: Query<&mut WatchConfettiTimer>, time: Res<Time>) {
    for mut timer in &mut q_timer {
        timer.tick(time.delta());
    }
}

fn handle_won_state_input(
    mut next_state: ResMut<NextState<AppState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut touch_events: EventReader<TouchInput>,
    q_timer: Query<&WatchConfettiTimer>,
) {
    if !q_timer.iter().any(|timer| timer.finished()) {
        return;
    }

    if keys.get_just_pressed().next().is_some()
        || mouse_buttons.get_just_released().next().is_some()
        || touch_events
            .read()
            .any(|event| event.phase == TouchPhase::Ended)
    {
        info!("Restarting game");
        next_state.set(AppState::Playing);
    }
}
