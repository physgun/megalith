//! UI display logic for representing [`Territory`] functions using the sickle_ui library.
//! In addition, some of the code design in this file is loosely copied from sickle_ui.

use bevy::prelude::*;
use sickle_ui::TrackedInteraction;

use crate::components_territory::*;
use crate::display_territory_sickle_customdraggable::*;

/// Extension trait for adding sickle_ui related functionality to Territory Tabs types.
pub trait SickleInterface {

}

/// Follow-up config for any [`Territory`] with [`DisplayLibrary::BevySickle`].
/// Runs after [`crate::display_territory::spawn_territory`].  
/// \
/// [`Territory`] must have stored the associated [`Entity`] IDs of a valid drag node and resize node representing it.
/// At least, it will have to, until entity relations gets here!
pub fn spawn_territory_sickle (
    mut commands: Commands,
    territory_query: Query<
    (Entity, &Territory, &DisplayLibrary, &Parent),
    Added<Territory>>
) {
    for (territory_entity, territory, display_library, territory_parent) in & territory_query {
        if matches!(display_library, DisplayLibrary::BevySickle) {

            let Some(drag_node_entity) = territory.drag_node() else {
                error!("Sickle spawner did not find associated drag node for Territory!");
                break;
            };
            let Some(resize_node_entity) = territory.resize_node() else {
                error!("Sickle spawner did not find associated resize node for Territory!");
                break;
            };

            commands.entity(drag_node_entity).insert((
                TrackedInteraction::default(), 
                CustomDraggable {
                    window_entity: territory_parent.get(),
                    ..default()
                }
            ));


        }
    }
}

/// Reads sickle_ui's [`Draggable`] component for a difference and creates a [`MoveRequest`] for the [`Territory`] if there's a diff.  
pub fn territory_move_request_sickle (
    mut commands: Commands,
    window_query: Query<
        (&Window, &Children),
        With<TerritoryTabs>
    >,
    territory_drag_query: Query<
        (Entity, &Territory, &DisplayLibrary)
    >,
    drag_node_query: Query<
        &CustomDraggable,
        (Changed<CustomDraggable>, With<TerritoryDragNode>)
    >
) {
    for (window, window_children) in & window_query {

        for (territory_entity, territory, display_library) in territory_drag_query.iter_many(window_children) {

            // This system will only process a Territory that is being represented by sickle.
            if !matches!(display_library, DisplayLibrary::BevySickle) {
                continue;
            }

            // Did someone forget to associate a drag node with this Territory?
            let Some(drag_node_entity) = territory.drag_node() else {
                warn!("Found a Territory without a drag node!");
                continue;
            };

            // Does this Territory have a Draggable drag node that was changed recently?
            let Ok(drag_data) = drag_node_query.get(drag_node_entity) else {
                continue;
            };

            // Is there a diff in the drag node's Draggable component? 
            let Some(drag_delta) = drag_data.diff else {
                continue;
            };

            // Is the diff greater than zero? Zero-size diffs can sneak in at drag end.
            if drag_delta == Vec2::ZERO { 
                continue; 
            }

            let new_move_request = MoveRequest {
                proposed_expanse: RectKit::from_screenspace(
                    Rect::from_center_size(
                        territory.expanse().screenspace().center() + drag_delta, 
                        territory.expanse().screenspace().size()
                    ),
                    window.width(), 
                    window.height()
                ),
                move_type: MoveRequestType::Drag
            };

            commands.entity(territory_entity).insert(new_move_request);

        }

    }
}