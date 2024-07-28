use bevy::prelude::*;

const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const TEXT_SPACING: f32 = 60.0;
const TEXT_OFFSET: f32 = 30.0;
const TEXT_SIZE: f32 = 50.0;

fn main() {
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

fn setup(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));

    commands.spawn(Camera2dBundle::default());

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
