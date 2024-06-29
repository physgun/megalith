//! UI display logic for representing [`Territory`] functions using the sickle_ui library.
//! In addition, some of the code design in this file is loosely copied from sickle_ui.

use bevy::{prelude::*, ui::RelativeCursorPosition};
use sickle_ui::{animated_interaction::AnimatedInteraction, drag_interaction::Draggable, interactions::InteractiveBackground, TrackedInteraction};

use crate::components_territory::*;

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
        (&Territory, &DisplayLibrary),
        Added<Territory>
    >,
    resize_grid_query: Query<&Children, With<TerritoryResizeGridNode>>,
    resize_button_query: Query<Entity, With<TerritoryResizeButtonNode>>
) {
    for (territory, display_library) in & territory_query {
        if matches!(display_library, DisplayLibrary::BevySickle) {

            let Some(drag_node_entity) = territory.drag_node() else {
                error!("Sickle spawner did not find associated drag node for Territory!");
                break;
            };
            let Some(resize_node_entity) = territory.resize_node() else {
                error!("Sickle spawner did not find associated resize node for Territory!");
                break;
            };
            let Ok(resize_grid_children) = resize_grid_query.get(resize_node_entity) else {
                error!("Sickle spawner did not find any resize grid children!");
                break;
            };

            // Sickle needs these components to track dragging.
            commands.entity(drag_node_entity).insert((
                TrackedInteraction::default(), 
                Draggable::default(),
                RelativeCursorPosition::default()
            ));

            // Resize buttons are just drag areas that change the size.
            for resize_button_entity in resize_button_query.iter_many(resize_grid_children) {
                commands.entity(resize_button_entity).insert((
                    TrackedInteraction::default(),
                    Draggable::default(),
                    RelativeCursorPosition::default(),
                    InteractiveBackground {
                        highlight: Color::rgb_u8(115, 235, 235).into(),
                        pressed: Color::rgb_u8(50, 245, 245).into(),
                        cancel: Color::NONE.into()
                    },
                    AnimatedInteraction::<InteractiveBackground>::default()
                ));
            }
        }
    }
}

/// Reads sickle_ui's [`Draggable`] component on the drag node for a difference and creates a [`MoveRequest`] for the [`Territory`].  
pub fn territory_drag_move_request_sickle (
    mut commands: Commands,
    window_query: Query<
        (&Window, &Children),
        With<TerritoryTabs>
    >,
    territory_drag_query: Query<
        (Entity, &Territory, &DisplayLibrary)
    >,
    drag_node_query: Query<
        &Draggable,
        (Changed<Draggable>, With<TerritoryDragNode>)
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


/// Reads sickle_ui's [`Draggable`] component on the resize node buttons for a difference and creates a [`MoveRequest`] for the [`Territory`].  
pub fn territory_resize_move_request_sickle (
    mut commands: Commands,
    window_query: Query<
        (&Window, &Children),
        With<TerritoryTabs>
    >,
    territory_resize_query: Query<
        (Entity, &Territory, &DisplayLibrary)
    >,
    resize_grid_children_query: Query<
        &Children,
        With<TerritoryResizeGridNode>
    >,
    resize_button_query: Query<
        (&Draggable, &ResizeDirection),
        (Changed<Draggable>, With<TerritoryResizeButtonNode>)
    >
) {
    for (window, window_children) in & window_query {

        for (territory_entity, territory, display_library) in territory_resize_query.iter_many(window_children) {

            // This system will only process a Territory that is being represented by sickle.
            if !matches!(display_library, DisplayLibrary::BevySickle) {
                continue;
            }

            // Is there a resize grid node addociated with this Territory?
            let Some(resize_grid_node) = territory.resize_node() else {
                warn!("Found a Territory without a resize grid node!");
                continue;
            };

            // Get the list of button entities for this Territory from its resize grid node's Children component.
            let Ok(resize_grid_children) = resize_grid_children_query.get(resize_grid_node) else {
                warn!("Territory's resize grid node has no children!");
                continue;
            };

            for (resize_button_draggable, resize_direction) in resize_button_query.iter_many(resize_grid_children) {

                // Is there a diff in the drag node's Draggable component? 
                let Some(drag_delta) = resize_button_draggable.diff else {
                    continue;
                };

                // Is the diff greater than zero? Zero-size diffs can sneak in at drag end.
                if drag_delta == Vec2::ZERO { 
                    continue; 
                }

                // Mod a new screenspace rect, depending on ResizeDirection. Everything is screenspace!
                let new_rect = resize_direction.add_delta_to_rect(territory.expanse().screenspace(), drag_delta);

                let new_move_request = MoveRequest {
                    proposed_expanse: RectKit::from_screenspace(
                        new_rect,
                        window.width(),
                        window.height()
                    ),
                    move_type: MoveRequestType::Resize(resize_direction.clone())
                };

                commands.entity(territory_entity).insert(new_move_request);
            }

        }

    }
}