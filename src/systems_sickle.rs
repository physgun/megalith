use bevy::prelude::*;
use sickle_ui::ui_builder::*;
use sickle_ui::widgets::container::UiContainerExt;

use crate::components_common::*;
use crate::components_ui::*;
use crate::events_ui::*;

pub fn spawn_territory_sickle (
    mut commands: Commands,
    mut territory_spawn_request_event: EventReader<TerritorySpawnRequest>
) {
    for spawn_event in territory_spawn_request_event.read() {
        if matches!(spawn_event.display_library, DisplayLibrary::BevySickle) {
            let new_territory = commands.spawn(
                (
                    Name("[TERRITORY] Spawned With Sickle".to_string()),
                    Territory {
                        screenspace_rect: spawn_event.screenspace_rect,
                        worldspace_rect: spawn_event.worldspace_rect,
                        ..Default::default()
                    },
                    SpatialBundle::default(),
                    DisplayLibrary::BevySickle
                )
            ).id();
            commands.entity(spawn_event.window_entity).add_child(new_territory);


        }
    }
}