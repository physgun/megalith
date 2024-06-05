use bevy::prelude::*;
use bevy::window::*;

use sickle_ui::ui_builder::UiBuilder;
use sickle_ui::ui_builder::UiBuilderExt;
use sickle_ui::ui_builder::UiRoot;
use sickle_ui::ui_style::*;
use sickle_ui::widgets::container::UiContainerExt;

use crate::components_territory::*;
use crate::systems_territory::*;

/// Custom [`Territory`] widget, implemented with extension traits, for displaying with sickle.
#[derive(Component)]
pub struct TerritoryWidgetSickle;

pub trait UiTerritorySickleWidgetExt<'w, 's> {
    fn territory_widget<'a>(&'a mut self) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> UiTerritorySickleWidgetExt<'w, 's> for UiBuilder<'w, 's, '_, UiRoot> {
    fn territory_widget<'a>(&'a mut self) -> UiBuilder<'w, 's, 'a, Entity> {
        self.spawn((NodeBundle::default(), TerritoryWidgetSickle))
    }
}

impl<'w, 's> UiTerritorySickleWidgetExt<'w, 's> for UiBuilder<'w, 's, '_, Entity> {
    fn territory_widget<'a>(&'a mut self) -> UiBuilder<'w, 's, 'a, Entity> {
        self.spawn((NodeBundle::default(), TerritoryWidgetSickle))
    }
}

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
            let new_territory = commands.spawn(
                (
                    Name::new("[TERRITORY] Sickle"),
                    Territory {
                        expanse: spawn_event.expanse,
                        ..Default::default()
                    },
                    SpatialBundle::default(),
                    DisplayLibrary::BevySickle
                )
            ).id();

            // Territory must hold the Node entities! Move things around!
            let territory_node = commands.spawn((
                Name::new("[NODE] Territory BG"),
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(spawn_event.expanse.relative_worldspace().min.x * 100.0),
                        top: Val::Percent(spawn_event.expanse.relative_worldspace().max.y * 100.0),
                        right: Val::Percent(spawn_event.expanse.relative_worldspace().max.x * 100.0),
                        bottom: Val::Percent(spawn_event.expanse.relative_worldspace().min.y * 100.0),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::GRAY),
                    ..default()
                }
            )).id();

            let mut ui_root = Entity::PLACEHOLDER;
            if let Ok(root) = window_ui_root_query.get(spawn_event.window_entity) {
                ui_root = root;
            }
            else {
                warn!("Broke. :)");
                break;
            }

            commands.entity(spawn_event.window_entity).add_child(new_territory);
            commands.entity(ui_root).add_child(territory_node);

        }
    }
}