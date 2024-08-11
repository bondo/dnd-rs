use bevy::{prelude::*, render::camera::ScalingMode};

const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const TEXT_SPACING: f32 = 60.0;
const TEXT_OFFSET: f32 = 30.0;
const TEXT_SIZE: f32 = 50.0;

fn main() {
    let level = level::Level::new();
    println!("{}", level);

    App::new()
        .add_plugins((DefaultPlugins, DungeonsAndDiagramsPlugin))
        .run();
}

pub struct DungeonsAndDiagramsPlugin;

impl Plugin for DungeonsAndDiagramsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .add_systems(Startup, setup)
            .add_systems(Update, (update_people, greet_people).chain());
    }
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

#[derive(Resource)]
struct GreetTimer(Timer);

#[derive(Component)]
struct Cell;

#[derive(Component)]
struct IsVisible(bool);

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Floor;

#[derive(Component)]
struct Treasure;

#[derive(Component)]
struct Monster;

fn setup(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));

    let mut camera_2d = Camera2dBundle::default();
    camera_2d.projection.scaling_mode = ScalingMode::Fixed {
        width: 1000.0,
        height: 1000.0,
    };
    camera_2d.transform = Transform::from_xyz(100.0, 100.0, 0.0);
    commands.spawn(camera_2d);

    for col in 1..10 {
        commands.spawn(
            TextBundle::from_sections([TextSection::new(
                col.to_string(),
                TextStyle {
                    font_size: TEXT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            )])
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(TEXT_OFFSET),
                left: Val::Px(TEXT_OFFSET + (col as f32) * TEXT_SPACING),
                ..default()
            }),
        );
    }

    for row in 1..10 {
        commands.spawn(
            TextBundle::from_sections([TextSection::new(
                row.to_string(),
                TextStyle {
                    font_size: TEXT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            )])
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(TEXT_OFFSET + (row as f32) * TEXT_SPACING),
                left: Val::Px(TEXT_OFFSET),
                ..default()
            }),
        );
    }
}

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    // update our timer with the time elapsed since the last update
    // if that caused the timer to finish, we say hello to everyone
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("hello {}!", name.0);
        }
    }
}

fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Elaina Proctor" {
            name.0 = "Elaina Hume".to_string();
            break; // We don’t need to change any other names
        }
    }
}

mod level {
    use std::fmt::Display;

    const SIZE: usize = 9;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Floor {
        Empty,
        Treasure,
        Monster,
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Cell {
        Any,
        Wall,
        Floor(Floor),
    }

    pub struct Level {
        cells: [[Cell; SIZE]; SIZE],
    }

    impl Level {
        pub fn new() -> Self {
            use rand::prelude::*;

            let mut cells = [[Cell::Any; SIZE]; SIZE];
            let mut rng = rand::thread_rng();

            let treasure_room_x = rng.gen::<usize>() % (SIZE - 2);
            let treasure_room_y = rng.gen::<usize>() % (SIZE - 2);

            // Fill treasure room with floor
            (treasure_room_x..treasure_room_x + 3).for_each(|x| {
                (treasure_room_y..treasure_room_y + 3).for_each(|y| {
                    cells[x][y] = Cell::Floor(Floor::Empty);
                });
            });

            let treasure_x = rng.gen::<usize>() % 3 + treasure_room_x;
            let treasure_y = rng.gen::<usize>() % 3 + treasure_room_y;
            cells[treasure_x][treasure_y] = Cell::Floor(Floor::Treasure);

            let mut potential_exits: Vec<(usize, usize)> = Vec::new();

            // Fill in walls around treasure room

            // Left side
            if treasure_room_x > 0 {
                (treasure_room_y..treasure_room_y + 3).for_each(|y| {
                    cells[treasure_room_x - 1][y] = Cell::Wall;
                    if treasure_room_x > 1 {
                        potential_exits.push((treasure_room_x - 1, y));
                    }
                })
            }

            // Right side
            if treasure_room_x + 3 < SIZE {
                (treasure_room_y..treasure_room_y + 3).for_each(|y| {
                    cells[treasure_room_x + 3][y] = Cell::Wall;
                    if treasure_room_x + 4 < SIZE {
                        potential_exits.push((treasure_room_x + 3, y));
                    }
                })
            }

            // Top side
            if treasure_room_y > 0 {
                (treasure_room_x..treasure_room_x + 3).for_each(|x| {
                    cells[x][treasure_room_y - 1] = Cell::Wall;
                    if treasure_room_y > 1 {
                        potential_exits.push((x, treasure_room_y - 1));
                    }
                })
            }

            // Bottom side
            if treasure_room_y + 3 < SIZE {
                (treasure_room_x..treasure_room_x + 3).for_each(|x| {
                    cells[x][treasure_room_y + 3] = Cell::Wall;
                    if treasure_room_y + 4 < SIZE {
                        potential_exits.push((x, treasure_room_y + 3));
                    }
                })
            }

            // Top left corner
            if treasure_room_x > 0 && treasure_room_y > 0 {
                cells[treasure_room_x - 1][treasure_room_y - 1] = Cell::Wall;
            }

            // Top right corner
            if treasure_room_x + 3 < SIZE && treasure_room_y > 0 {
                cells[treasure_room_x + 3][treasure_room_y - 1] = Cell::Wall;
            }

            // Bottom left corner
            if treasure_room_x > 0 && treasure_room_y + 3 < SIZE {
                cells[treasure_room_x - 1][treasure_room_y + 3] = Cell::Wall;
            }

            // Bottom right corner
            if treasure_room_x + 3 < SIZE && treasure_room_y + 3 < SIZE {
                cells[treasure_room_x + 3][treasure_room_y + 3] = Cell::Wall;
            }

            let (exit_x, exit_y) = potential_exits[rng.gen::<usize>() % potential_exits.len()];
            cells[exit_x][exit_y] = Cell::Floor(Floor::Empty);

            let mut expand_from = vec![(exit_x, exit_y)];
            while let Some((x, y)) = expand_from.pop() {
                let mut free_neighbours: Vec<(usize, usize)> = Vec::new();

                let top_left_good = x == 0
                    || y == 0
                    || cells[x - 1][y - 1] == Cell::Any
                    || cells[x - 1][y - 1] == Cell::Wall;

                let top_right_good = x + 1 == SIZE
                    || y == 0
                    || cells[x + 1][y - 1] == Cell::Any
                    || cells[x + 1][y - 1] == Cell::Wall;

                let bottom_left_good = x == 0
                    || y + 1 == SIZE
                    || cells[x - 1][y + 1] == Cell::Any
                    || cells[x - 1][y + 1] == Cell::Wall;

                let bottom_right_good = x + 1 == SIZE
                    || y + 1 == SIZE
                    || cells[x + 1][y + 1] == Cell::Any
                    || cells[x + 1][y + 1] == Cell::Wall;

                // Move up
                if y > 0 && top_left_good && top_right_good && cells[x][y - 1] == Cell::Any {
                    free_neighbours.push((x, y - 1));
                }

                // Move down
                if y + 1 < SIZE
                    && bottom_left_good
                    && bottom_right_good
                    && cells[x][y + 1] == Cell::Any
                {
                    free_neighbours.push((x, y + 1));
                }

                // Move left
                if x > 0 && top_left_good && bottom_left_good && cells[x - 1][y] == Cell::Any {
                    free_neighbours.push((x - 1, y));
                }

                // Move right
                if x + 1 < SIZE
                    && top_right_good
                    && bottom_right_good
                    && cells[x + 1][y] == Cell::Any
                {
                    free_neighbours.push((x + 1, y));
                }

                // Ignore a random number of neighbours
                if free_neighbours.len() > 1 {
                    free_neighbours.shuffle(&mut rng);
                    let num_drop = rng.gen_range(0..free_neighbours.len());
                    (0..num_drop).for_each(|_| {
                        free_neighbours.pop();
                    });
                }

                // Put floor on remaining neighbours and queue expansion
                free_neighbours.into_iter().for_each(|(x, y)| {
                    cells[x][y] = Cell::Floor(Floor::Empty);
                    expand_from.push((x, y));
                });

                // Shuffle queue
                expand_from.shuffle(&mut rng);
            }

            // Replace remaining Any cells with walls
            (0..SIZE).for_each(|x| {
                (0..SIZE).for_each(|y| {
                    if cells[x][y] == Cell::Any {
                        cells[x][y] = Cell::Wall;
                    }
                })
            });

            // TODO: Fill in monsters

            Level { cells }
        }
    }

    impl Display for Level {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for y in 0..SIZE {
                for x in 0..SIZE {
                    let cell = match self.cells[x][y] {
                        Cell::Any => '?',
                        Cell::Wall => '#',
                        Cell::Floor(Floor::Empty) => '.',
                        Cell::Floor(Floor::Treasure) => 'T',
                        Cell::Floor(Floor::Monster) => 'M',
                    };
                    write!(f, "{}", cell)?;
                }
                writeln!(f)?;
            }
            Ok(())
        }
    }
}
