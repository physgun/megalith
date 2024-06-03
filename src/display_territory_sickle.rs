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
                
                let mut ui_camera_entity = Entity::PLACEHOLDER;
                for (camera_entity, camera_parent) in & ui_camera_query {
                    if camera_parent.get() == event.window {
                        ui_camera_entity = camera_entity;
                        debug!("UI Cam found!");
                    }
                }

                let mut root_entity = Entity::PLACEHOLDER;
                commands.ui_builder(UiRoot).container((
                    Name::new("[ROOT NODE] Sickle Root Node"),
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        ..default()
                    },
                    TargetCamera(ui_camera_entity),
                ), |container| {
                    root_entity = container
                        .spawn((
                            Name::new("[ROOT NODE] Territory Tabs Root Node"),
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::DARK_GRAY),
                                ..default()
                            },
                            TerritoryTabsUIRoot
                        ))
                        .id();
                });

                commands.entity(event.window).add_child(root_entity);
                debug!("Root entity added as child to Window!");
            }
        }
    }
}

pub fn spawn_territory_sickle (
    mut commands: Commands,
    mut territory_spawn_request_event: EventReader<TerritorySpawnRequest>,
    window_ui_root_query: Query<(Entity, &Parent), With<TerritoryTabsUIRoot>>
) {
    for spawn_event in territory_spawn_request_event.read() {
        if matches!(spawn_event.display_library, DisplayLibrary::BevySickle) {
            let new_territory = commands.spawn(
                (
                    Name::new("[TERRITORY] Spawned With Sickle"),
                    Territory {
                        screenspace_rect: spawn_event.screenspace_rect,
                        worldspace_rect: spawn_event.worldspace_rect,
                        ..Default::default()
                    },
                    SpatialBundle::default(),
                    DisplayLibrary::BevySickle
                )
            ).id();

            let mut root_entity = Entity::PLACEHOLDER;
            if let Ok((entity, root_parent)) = window_ui_root_query.get(spawn_event.window_entity) {
                if root_parent.get() == spawn_event.window_entity {
                    root_entity = entity;
                }
            }

            commands.entity(spawn_event.window_entity).add_child(new_territory);

        }
    }
}