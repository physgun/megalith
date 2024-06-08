//! UI display logic for displaying a [`Territory`] with bevy_ui.

use bevy::prelude::*;

use crate::components_territory::*;
use crate::systems_territory::*;

/// Trait extension for the [`Territory`] component, so I can move all the verbose [`Node`] stuff into its own module. 
pub trait TerritoryNodes{
    fn base_node_template(&self) -> impl Bundle;
    fn border_node_template(&self) -> impl Bundle;
    fn drag_node_template(&self) -> impl Bundle;
    fn resize_node_template(&self) -> impl Bundle;
}

impl TerritoryNodes for Territory {

    /// Returns a [`Bundle`] of a template, named, base [`Node`]. A background the exact size of the [`Territory`].  
    /// \
    /// Note: This [`Node`] needs the [`Territory`] to have a complete [`RectKit`]!
    fn base_node_template(&self) -> impl Bundle {
        (
            Name::new("[NODE] Territory Base Node"),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(self.expanse.relative_screenspace.width() * 100.0),
                    height: Val::Percent(self.expanse.relative_screenspace.height() * 100.0),
                    left: Val::Percent(self.expanse.relative_screenspace.min.x * 100.0),
                    top: Val::Percent(self.expanse.relative_screenspace.min.y * 100.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
                background_color: BackgroundColor(Color::rgb_u8(223, 168, 120)),
                focus_policy: bevy::ui::FocusPolicy::Block,
                ..default()
            }
        )
    }

    /// Returns a [`Bundle`] of a template, named, border [`Node`] representing the visual borders of the [`Territory`].  
    /// \
    /// We have borders as a separate node to allow the resize drag buttons to sit on top of them
    /// without using up an [`Outline`] component.
    fn border_node_template(&self) -> impl Bundle {
        (
            Name::new("[NODE] Territory Border Node"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                border_color: BorderColor(Color::rgb_u8(206, 230, 243)),
                ..default()
            }
        )
    }

    /// Returns a [`Bundle`] of a template, named, drag [`Node`].  
    /// \
    /// This will be the area of the [`Territory`] that will drag it around.
    /// Note that native Bevy UI does not have drag or resize interactions, 
    /// so that functionality will have to be added by a third party crate.
    fn drag_node_template(&self) -> impl Bundle {
        (
            Name::new("[NODE] Territory Drag Node"),
            ButtonBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                ..default()
            }
        )
    }

    /// Returns a [`Bundle`] of a template, named, grid [`Node`] for the resize buttons.  
    /// \
    /// A simple 3 x 3 CSS Grid for placing the eight resize directions and a central content area.
    fn resize_node_template(&self) -> impl Bundle {
        let resize_grid = vec![
            GridTrack::px(4.0),
            GridTrack::flex(1.0),
            GridTrack::px(4.0)
        ];
        (
            Name::new("[Node] Territory Resize Grid Node"),
            NodeBundle {
                style: Style {
                    display: Display::Grid,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    grid_template_rows: resize_grid.clone(),
                    grid_template_columns: resize_grid.clone(),
                    row_gap: Val::Px(1.0),
                    column_gap: Val::Px(1.0),
                    ..default()
                },
                z_index: ZIndex::Local(10), // Grid needs to sit on top of both the border and the drag node.
                ..default()
            }
        )
    }

}

/// The first system to respond to a [`TerritorySpawnRequest`]. Actually spawns the [`Territory`] entity and associated components.
/// This system also spawns bevy_ui nodes for UI representation, which subsequent library-specific spawn systems can build on top of.
pub fn spawn_territory (
    mut commands: Commands,
    mut territory_spawn_request_event: EventReader<TerritorySpawnRequest>,
    root_node_query: Query<(Entity, &TerritoryTabsUIRoot)>
) {
    for spawn_event in territory_spawn_request_event.read() {
        
        // Spawn new Territory with the requested RectKit.
        let mut new_territory = Territory::empty();
        new_territory.expanse = spawn_event.expanse;

        // Find the correct bevy_ui root node entity associated with our spawn event window entity.
        // This is messy and should be refactored when Bevy's entity relations features arrive.
        let mut root_node_entity = Entity::PLACEHOLDER;
        for (ui_root_entity, ui_root_window) in & root_node_query {
            if ui_root_window.associated_window_entity == spawn_event.window_entity {
                root_node_entity = ui_root_entity;
            }
        }

        // Again, entity relations should render this unnecessary in the future.
        if root_node_entity == Entity::PLACEHOLDER {
            error!("Unable to find [ROOT NODE] entity for this window, Territory spawn canceled!");
            break;
        }
        
        // If the entire Territory UI is being handled by egui's immediate mode library, then no nodes are required.
        // For all others, spawn the needed node entities and stash the needed entity IDs in the Territory component.
        let base_node_option;
        let drag_node_option;
        let resize_node_option;
        match spawn_event.display_library {
            DisplayLibrary::BevyEgui => { 
                base_node_option = None;
                drag_node_option = None;
                resize_node_option = None;
            },
            DisplayLibrary::BevyUi | 
            DisplayLibrary::BevySickle => {
                let base_node_entity = commands.spawn(new_territory.base_node_template()).id();
                let border_node_entity = commands.spawn(new_territory.border_node_template()).id();
                let drag_node_entity = commands.spawn(new_territory.drag_node_template()).id();
                let resize_node_entity = commands.spawn(new_territory.resize_node_template()).id();

                commands.entity(base_node_entity).add_child(border_node_entity);
                commands.entity(base_node_entity).add_child(resize_node_entity);
                commands.entity(border_node_entity).add_child(drag_node_entity);

                base_node_option = Some(base_node_entity);
                drag_node_option = Some(drag_node_entity);
                resize_node_option = Some(resize_node_entity);
            }
        }
        new_territory.base_node = base_node_option;
        new_territory.drag_node = drag_node_option;
        new_territory.resize_node = resize_node_option;

        // Spawn Territory.
        let new_territory_entity = commands.spawn(
            (
                Name::new("[TERRITORY] Base"),
                new_territory,
                SpatialBundle::default(),
                spawn_event.display_library
            )
        ).id();

        // Add new Territory to the spawn Window.
        commands.entity(spawn_event.window_entity).add_child(new_territory_entity);

        // If we have a base node entity to represent the Territory with, 
        // add it as a child of the root node entity associated with the window.
        if base_node_option.is_some() { 
            commands.entity(root_node_entity).add_child(base_node_option.unwrap());
        }
    }
}

/// Handles all [`TerritoryDespawnRequest`], cleaning up the [`Territory`] and all associated nodes.
pub fn despawn_territory (
    mut commands: Commands,
    mut territory_despawn_request_event: EventReader<TerritoryDespawnRequest>,
    territory_query: Query<&Territory>
) {
    for despawn_event in territory_despawn_request_event.read() {
        if let Ok(despawning_territory) = territory_query.get(despawn_event.despawned_territory) {
            // Despawn base UI Node, if it exists.
            if despawning_territory.base_node().is_some() {
                commands.entity(despawning_territory.base_node.unwrap()).despawn_recursive();
            }
            // Despawn Territory.
            commands.entity(despawn_event.despawned_territory).despawn_recursive();
        }
    }
}