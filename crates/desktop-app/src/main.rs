#![windows_subsystem = "windows"]

use bevy::prelude::*;
use dnd_rs_plugin::DungeonsAndDiagramsPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // To update asset meta run with `cargo run --features bevy/asset_processor`
            // DefaultPlugins.set(AssetPlugin {
            //     mode: AssetMode::Processed,
            //     ..Default::default()
            // }),
            DungeonsAndDiagramsPlugin,
        ))
        .run();
}
