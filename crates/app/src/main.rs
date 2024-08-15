use bevy::prelude::*;
use dnd_rs_plugin::DungeonsAndDiagramsPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DungeonsAndDiagramsPlugin))
        .run();
}
