use bevy::prelude::*;
use bevy::window::*;

use sickle_ui::ui_builder::UiBuilder;
use sickle_ui::ui_builder::UiBuilderExt;
use sickle_ui::ui_builder::UiRoot;
use sickle_ui::ui_style::*;
use sickle_ui::widgets::container::UiContainerExt;

use crate::components_territory::*;
use crate::systems_territory::*;
use crate::display_territory::*;

/// Follow-up config for any [`Window`] with [`DisplayLibrary::BevySickle`].
/// Summoned by a [`WindowCreated`] event and configures that exact window.
/// Must run after the default [`configure_os_window`].
pub fn configure_os_window_sickle (
    mut commands: Commands,
    mut window_spawn_detected_events: EventReader<WindowCreated>,
    window_query: Query<
        &DisplayLibrary,
        (With<TerritoryTabs>, With<Window>)
        >,
    ui_camera_query: Query<
        (Entity, &Parent), 
        With<TerritoryTabsCamera>
        >
) {
    for event in window_spawn_detected_events.read() {
        if let Ok(display_library) = window_query.get(event.window){
            if matches!(display_library, DisplayLibrary::BevySickle) {
                
            }
        }
    }
}

pub fn spawn_territory_sickle (
    mut commands: Commands,
    mut territory_spawn_request_event: EventReader<TerritorySpawnRequest>,
    window_ui_root_query: Query<Entity, With<TerritoryTabsUIRoot>>
) {
    for spawn_event in territory_spawn_request_event.read() {
        if matches!(spawn_event.display_library, DisplayLibrary::BevySickle) {

        }
    }
}