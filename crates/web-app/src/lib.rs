mod utils;

use bevy::prelude::*;
use dnd_rs_plugin::DungeonsAndDiagramsPlugin;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run() {
    utils::set_panic_hook();

    App::new()
        .add_plugins((DefaultPlugins, DungeonsAndDiagramsPlugin))
        .run();
}
