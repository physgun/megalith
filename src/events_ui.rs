use bevy::prelude::*;


// Sent when a territory gets dragged, with the drag information.
#[derive(Event)]
pub struct TerritoryDragged {
    pub dragged_entity: Entity,
    pub mouse_delta: Vec2
}
