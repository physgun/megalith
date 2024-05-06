use bevy::prelude::*;

// Sent when a Territory starts being dragged.
#[derive(Event)]
pub struct TerritoryDragStarted;

// Sent when a Territory is being dragged, with the drag information.
#[derive(Event)]
pub struct TerritoryDragged {
    pub window_entity: Entity,
    pub territory_entity: Entity,
    pub mouse_delta: Vec2,
}

// Sent when a Territory stops being dragged.
#[derive(Event)]
pub struct TerritoryDragEnded;

// Sent when a Territory starts being resized. 
#[derive(Event)]
pub struct TerritoryResizeStarted;

// Sent when a Territory is being resized, with the resize information.
#[derive(Event)]
pub struct TerritoryResizing {
    pub window_entity: Entity,
    pub territory_entity: Entity,
    pub delta_size: Vec2,
}

// Sent when a Territory is no longer being resized.
#[derive(Event)]
pub struct TerritoryResizeEnded;