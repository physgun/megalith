//! UI display logic for representing [`Territory`] functions using the sickle_ui library.
//! In addition, much of the code in this file is copied from sickle_ui and repurposed to suite the context of the project.

use bevy::prelude::*;
use sickle_ui::{drag_interaction::Draggable, TrackedInteraction};

use crate::components_territory::*;

/// Follow-up config for any [`Territory`] with [`DisplayLibrary::BevySickle`].
/// Runs after [`crate::display_territory::spawn_territory`].  
/// \
/// [`Territory`] must have stored the associated [`Entity`] IDs of a valid drag node and resize node representing it.
/// At least, it will have to, until entity relations gets here!
pub fn spawn_territory_sickle (
    mut commands: Commands,
    mut territory_query: Query<
    (Entity, &Territory, &DisplayLibrary),
    Added<Territory>>,
    territory_drag_node_query: Query<(Entity, &TerritoryDragNode)>,
    territory_resize_node_query: Query<(Entity, &TerritoryResizeGridNode)>,
) {
    for (mut territory_entity, mut territory, display_library) in & territory_query {
        if matches!(display_library, DisplayLibrary::BevySickle) {

            let Some(drag_node_entity) = territory.drag_node() else {
                error!("Sickle spawner did not find associated drag node for Territory!");
                break;
            };
            let Some(resize_node_entity) = territory.resize_node() else {
                error!("Sickle spawner did not find associated resize node for Territory!");
                break;
            };

            commands.entity(drag_node_entity).insert((TrackedInteraction::default(), Draggable::default()));


        }
    }
}