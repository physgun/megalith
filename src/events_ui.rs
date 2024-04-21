use bevy::prelude::*;

// Sent when a Territory starts being dragged.
#[derive(Event)]
pub struct TerritoryDragStarted;

// Sent when a Territory is being dragged, with the drag information.
#[derive(Event)]
pub struct TerritoryDragged {
    pub window_entity: Entity,
    pub dragged_entity: Entity,
    pub mouse_delta: Vec2
}

// Sent when a Territory stops being dragged.
#[derive(Event)]
pub struct TerritoryDragEnded;