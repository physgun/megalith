//! UI display logic for displaying a [`Territory`] with bevy_ui.

use bevy::prelude::*;

use crate::components_territory::*;
use crate::systems_territory::*;

/// Trait extension for the [`Territory`] component, so I can move all the verbose [`Node`] stuff into its own module. 
pub trait TerritoryNodes{
    fn base_node_template(&self) -> impl Bundle;
    fn drag_node_template(&self) -> impl Bundle;
    fn resize_grid_template(&self) -> impl Bundle;
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
                    border: UiRect::all(Val::Px(2.0)),
                    width: Val::Percent(self.expanse.relative_screenspace.width() * 100.0),
                    height: Val::Percent(self.expanse.relative_screenspace.height() * 100.0),
                    left: Val::Percent(self.expanse.relative_screenspace.min.x * 100.0),
                    top: Val::Percent(self.expanse.relative_screenspace.min.y * 100.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
                background_color: BackgroundColor(Color::DARK_GRAY),
                border_color: BorderColor(Color::GRAY),
                focus_policy: bevy::ui::FocusPolicy::Block,
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
    /// 
    fn resize_grid_template(&self) -> impl Bundle {
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
                z_index: ZIndex::Local(10),
                ..default()
            }
        )
    }

}

/// Add docs!
pub fn spawn_territory (
    mut commands: Commands,
    mut territory_spawn_request_event: EventReader<TerritorySpawnRequest>,
    root_node_query: Query<(Entity, &TerritoryTabsUIRoot)>
) {
    for spawn_event in territory_spawn_request_event.read() {
        
        let mut new_territory = Territory::empty();
        new_territory.expanse = spawn_event.expanse;

        let mut root_node_entity = Entity::PLACEHOLDER;
        for (ui_root_entity, ui_root_window) in & root_node_query {
            if ui_root_window.associated_window_entity == spawn_event.window_entity {
                root_node_entity = ui_root_entity;
            }
        }
        
        let base_node_entity;
        match spawn_event.display_library {
            DisplayLibrary::BevyEgui => { 
                base_node_entity = None;
            },
            DisplayLibrary::BevyUi | DisplayLibrary::BevySickle => {
                base_node_entity = Some(commands.spawn(new_territory.base_node_template()).id());
            }
        }
        new_territory.base_node = base_node_entity;

        let new_territory_entity = commands.spawn(
            (
                Name::new("[TERRITORY] Base"),
                new_territory,
                SpatialBundle::default(),
                spawn_event.display_library
            )
        ).id();

        commands.entity(spawn_event.window_entity).add_child(new_territory_entity);
        if base_node_entity.is_some() { 
            commands.entity(root_node_entity).add_child(base_node_entity.unwrap());
        }
    }
}