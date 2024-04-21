use bevy::prelude::*;

/// Global resource for getting the mouse position in Bevy's 2D camera space.
/// From screenspace (upper left is origin and y goes down),
/// To default camera (center is origin and y goes up).
/// Also stores the window, territory, and tab entity hovered over, if applicable.
#[derive(Resource)]
pub struct WorldMousePosition {
    pub pos: Vec2,
    pub window: Option<Entity>,
    pub territory: Option<Entity>,
    pub tab: Option<Entity>
}
impl Default for WorldMousePosition {
    fn default() -> Self {
        WorldMousePosition {
            pos: Vec2::new(0.0, 0.0),
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
    pub tab_offset: Vec2
}
impl Default for TerritorySettings{
    fn default() -> Self {
        TerritorySettings {
            min_size: Vec2 {x: 50.0, y: 50.0},
            default_size: Vec2 {x: 250.0, y: 250.0},
            tab_offset: Vec2{x: 5.0, y: 5.0}
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
            min_size: Vec2{x: 20.0, y: 20.0}
        }
    }
}