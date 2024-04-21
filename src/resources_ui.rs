use bevy::prelude::*;

/// Global resource for getting the mouse position in Bevy's 2D camera space.
/// In Screenspace (upper left is origin and y goes down),
/// And Worldspace (center is origin and y goes up).
/// Also stores the window, territory, and tab entity hovered over, if applicable.
#[derive(Resource)]
pub struct WorldMousePosition {
    pub screenspace_pos: Vec2,
    pub worldspace_pos: Vec2,
    pub window: Option<Entity>,
    pub territory: Option<Entity>,
    pub tab: Option<Entity>
}
impl Default for WorldMousePosition {
    fn default() -> Self {
        WorldMousePosition {
            screenspace_pos: Vec2::new(0.0, 0.0),
            worldspace_pos: Vec2::new(0.0, 0.0),
            window: None,
            territory: None,
            tab: None
        }
    }
}

// Config stuff for Territories
#[derive(Resource)]
pub struct TerritorySettings {
    pub min_size: Vec2,
    pub default_size: Vec2,
    pub inner_margins: Vec2,
    pub spacing: f32
}
impl Default for TerritorySettings{
    fn default() -> Self {
        TerritorySettings {
            min_size: Vec2 {x: 50.0, y: 25.0},
            default_size: Vec2 {x: 250.0, y: 250.0},
            inner_margins: Vec2{x: 2.5, y: 2.5},
            spacing: 2.5
        }
    }
}
 // Config stuff for Tabs
#[derive(Resource)]
pub struct TabSettings {
    pub min_size: Vec2
}
impl Default for TabSettings {
    fn default() -> Self {
        TabSettings {
            min_size: Vec2{x: 20.0, y: 15.0}
        }
    }
}