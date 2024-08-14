use bevy::{prelude::*, render::camera::ScalingMode};

use dnd_rs_level::Level;

const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const TEXT_SPACING: f32 = 60.0;
const TEXT_OFFSET: f32 = 30.0;
const TEXT_SIZE: f32 = 50.0;

pub struct DungeonsAndDiagramsPlugin;

impl Plugin for DungeonsAndDiagramsPlugin {
    fn build(&self, app: &mut App) {
        let level = Level::random(9, 9);
        println!("{:?}", level);

        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .insert_resource(level)
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

fn setup(mut commands: Commands, level: Res<Level>) {
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

    let mut column_headers = vec![0; level.width()];
    let mut row_headers = vec![0; level.height()];
    level.iter().for_each(|c| {
        if c.has_wall() {
            column_headers[c.x()] += 1;
            row_headers[c.y()] += 1;
        }

        // TODO: Spawn monsters and treasure
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
