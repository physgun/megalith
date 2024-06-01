use bevy::prelude::*;

use crate::components_ui::*;

/// Sent when a UI element is issued a  [`MoveRequest`] component.
#[derive(Event)]
pub struct MoveRequestApplied;

/// Sent when a system has commanded a [`Territory`] to spawn.
#[derive(Event)]
pub struct TerritorySpawnRequest {
    pub window_entity: Entity,
    pub screenspace_rect: Rect,
    pub worldspace_rect: Rect,
    pub display_library: DisplayLibrary
}