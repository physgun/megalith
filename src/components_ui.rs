use bevy::prelude::*;



// Territories and Tabs can have a horizontal or vertical orientation.
#[derive(Default)]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal
}

// Identifies entity as a Territory that contains Tabs. Typically needs Tabs, and will be cleaned up if it has none.
// Territories will manage Tabs through Parent / Child components.
#[derive(Component)]
pub struct Territory {
    pub rect: Rect,
    pub orientation: Orientation
}
impl Default for Territory {
    fn default() -> Self {
        Territory {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0), 
            orientation: Orientation::Vertical}
    }
}
impl Territory {
    pub fn new(rect: Rect, orientation: Orientation) -> Self {
        Territory {rect, orientation}
    }
}

// Identifies entity as a Tab, which can be active or inactive, and represent a type of UI.
#[derive(Component)]
pub struct Tab {
    pub active: bool,
    pub name: String, 
    pub icon: char, 
    pub tab_type: TabType,
}
impl Default for Tab {
    fn default() -> Self {
        Tab {
            active: false,
            name: "DEFAULT TAB".to_string(), 
            icon: '⚠',
            tab_type: TabType::FileSystem,
        }
    }
}
impl Tab {
    pub fn build(active: bool, name: String, icon: char, tab_type: TabType) -> Self {
        Tab {active, name, icon, tab_type}
    }

    pub fn build_from_type(tab_type: TabType) -> Self {
        match tab_type {
            TabType::FileSystem => Tab {name: "File".to_string(), icon: '📁', tab_type, ..Default::default()},        
            TabType::DevBox => Tab {name: "Dev Box".to_string(), icon: '🛠', tab_type, ..Default::default()},
            TabType::ECS => Tab {name: "ECS".to_string(), icon: '🍱', tab_type, ..Default::default()},
            TabType::Glossary => Tab {name: "Glossary".to_string(), icon: '📖', tab_type, ..Default::default()},
            TabType::SiteView => Tab {name: "Site View".to_string(), icon: '👁', tab_type, ..Default::default()},
            }
    }
}

pub enum TabType {
    FileSystem,
    DevBox,
    ECS,
    Glossary,
    SiteView,
}

// Denotes entity as visual assistant for visualizing the placement of things.
// Also used to validate the spawn location of said things.
#[derive(Component)]
pub struct Placeholder {
    pub placeholder_type: PlaceholderType, 
    pub valid_spawn: bool,
    pub visual_rects: Vec<Rect>
}
impl Default for Placeholder {
    fn default() -> Self {
        Placeholder {
            placeholder_type: PlaceholderType::SpawnTerritory,
            valid_spawn: false,
            visual_rects: vec![
                Rect::new(0.0, 0.0, 100.0, 100.0),
                Rect::new(0.0, 0.0, 300.0, 300.0)
            ]
        }
    }
}

pub enum PlaceholderType {
    SpawnTerritory,
    TabMove,
    TabOrigin,
    SpawnWindow,
    CombineTerritories,
    LoadLayout
}

/// Marker component configuring a window to use Territory Tabs UI.
#[derive(Component)]
pub struct TerritoryTabsUI;

/// Marker component denoting that this Territory Tabs window will use the egui library.
#[derive(Component)]
pub struct EguiDisplay;

/// This marks a camera as being intended for use as a 2D world UI background camera.
/// Mouse seeking systems will check cameras with this component.
#[derive(Component)]
pub struct MouseSeekingCamera;
