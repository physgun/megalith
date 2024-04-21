use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use megalith::ui::*;

fn main() {

    App::new()
        .add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            level: bevy::log::Level::INFO,
            filter: "wgpu=warn,bevy_ecs=warn,megalith=debug".to_string(),
            ..Default::default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(TerritoryTabsPlugin)
        .run();

}

