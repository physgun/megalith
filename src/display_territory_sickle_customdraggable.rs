//! Custom Draggable copy/paste/hack to get around MacOS' dislike of the sickle_ui::drag_interaction.rs use of CursorGrab.
//! Once the original Draggable is refactored, this file will be removed.  
//! \
//! Basically just appends `Custom` to the front of everything so I remember it's different,
//! and removes the PrimaryWindow dependency by stashing the Node's window in another field.  
//! \
//! \
//! Oh yeah, and the cursor grab update system is gone.

use bevy::prelude::*;
use bevy::reflect::Reflect;

use sickle_ui::{FluxInteraction, FluxInteractionUpdate};

use crate::systems_territory::TerritoryUpdateMotion;

pub struct CustomDragInteractionPlugin;

impl Plugin for CustomDragInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, CustomDraggableUpdate.after(FluxInteractionUpdate).before(TerritoryUpdateMotion))
            .add_systems(
                Update,
                (
                    custom_update_drag_progress,
                    custom_update_drag_state
                )
                    .chain()
                    .in_set(CustomDraggableUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct CustomDraggableUpdate;

// Entity has no default, so we need to implement our own.
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct CustomDraggable {
    pub window_entity: Entity,
    pub state: CustomDragState,
    pub origin: Option<Vec2>,
    pub position: Option<Vec2>,
    pub diff: Option<Vec2>,
    pub source: CustomDragSource,
}

impl Default for CustomDraggable {
    fn default() -> Self {
        CustomDraggable {
            window_entity: Entity::PLACEHOLDER,
            state: CustomDragState::default(),
            origin: None,
            position: None,
            diff: None,
            source: CustomDragSource::default()
        }
    }
}

impl CustomDraggable {
    fn clear(&mut self) {
        self.origin = None;
        self.position = None;
        self.diff = Vec2::default().into();
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
#[reflect]
pub enum CustomDragState {
    #[default]
    Inactive,
    MaybeDragged,
    DragStart,
    Dragging,
    DragEnd,
    DragCanceled,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
#[reflect]
pub enum CustomDragSource {
    #[default]
    Mouse,
    Touch(u64),
}

fn custom_update_drag_progress(
    mut q_draggable: Query<(&mut CustomDraggable, &FluxInteraction)>,
    q_window: Query<&Window>,
    r_touches: Res<Touches>,
    r_keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut draggable, flux_interaction) in &mut q_draggable {
        let Ok(window) = q_window.get(draggable.window_entity) else {
            continue;
        };

        if draggable.state == CustomDragState::DragEnd {
            draggable.state = CustomDragState::Inactive;
            draggable.clear();
        } else if draggable.state == CustomDragState::DragCanceled {
            draggable.state = CustomDragState::Inactive;
        } else if *flux_interaction == FluxInteraction::Pressed
            && (draggable.state == CustomDragState::MaybeDragged
                || draggable.state == CustomDragState::DragStart
                || draggable.state == CustomDragState::Dragging)
        {
            if (draggable.state == CustomDragState::DragStart || draggable.state == CustomDragState::Dragging)
                && r_keys.just_pressed(KeyCode::Escape)
            {
                draggable.state = CustomDragState::DragCanceled;
                draggable.clear();
                continue;
            }

            // Drag start is only a single frame, triggered after initial movement
            if draggable.state == CustomDragState::DragStart {
                draggable.state = CustomDragState::Dragging;
            }

            let position: Option<Vec2> = match draggable.source {
                CustomDragSource::Mouse => window.cursor_position(),
                CustomDragSource::Touch(id) => match r_touches.get_pressed(id) {
                    Some(touch) => touch.position().into(),
                    None => None,
                },
            };

            if let (Some(current_position), Some(new_position)) = (draggable.position, position) {
                let diff = new_position - current_position;

                // No tolerance threshold, just move
                if diff.length_squared() > 0. {
                    if draggable.state == CustomDragState::MaybeDragged {
                        draggable.state = CustomDragState::DragStart;
                    }

                    draggable.position = new_position.into();
                    draggable.diff = (new_position - current_position).into();
                }
            }
        }
    }
}

fn custom_update_drag_state(
    mut q_draggable: Query<(&mut CustomDraggable, &FluxInteraction), Changed<FluxInteraction>>,
    q_window: Query<&Window>,
    r_touches: Res<Touches>,
) {
    for (mut draggable, flux_interaction) in &mut q_draggable {
        if *flux_interaction == FluxInteraction::Pressed
            && draggable.state != CustomDragState::MaybeDragged
        {
            let mut drag_source = CustomDragSource::Mouse;
            let Ok(window) = q_window.get(draggable.window_entity) else {
                continue;
            };
            let mut position = window.cursor_position();
            if position.is_none() {
                position = r_touches.first_pressed_position();
                drag_source = CustomDragSource::Touch(r_touches.iter().next().unwrap().id());
            }

            draggable.state = CustomDragState::MaybeDragged;
            draggable.source = drag_source;
            draggable.origin = position;
            draggable.position = position;
            draggable.diff = Vec2::default().into();
        } else if *flux_interaction == FluxInteraction::Released
            || *flux_interaction == FluxInteraction::PressCanceled
        {
            if draggable.state == CustomDragState::DragStart || draggable.state == CustomDragState::Dragging {
                draggable.state = CustomDragState::DragEnd;
            } else if draggable.state == CustomDragState::MaybeDragged {
                draggable.state = CustomDragState::Inactive;
                draggable.clear();
            }
        }
    }
}