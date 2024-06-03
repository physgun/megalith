use bevy::prelude::*;

/// Marks a `Territory` as being a visual overlay. Any `Territory` marked with this won't collide with other `Territory`s.
/// Used as a visual guide to UI behavior.
#[derive(Component)]
pub struct Overlay;

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
    pub screenspace_visual_rects: Vec<Rect>,
    pub worldspace_visual_rects: Vec<Rect>
}
impl Default for Placeholder {
    fn default() -> Self {
        Placeholder {
            placeholder_type: PlaceholderType::SpawnTerritory,
            valid_spawn: false,
            screenspace_visual_rects: vec![
                Rect::new(0.0, 0.0, 100.0, 100.0),
                Rect::new(0.0, 0.0, 300.0, 300.0)
            ],
            worldspace_visual_rects: vec![
                Rect::new(0.0, 0.0, 100.0, -100.0),
                Rect::new(0.0, 0.0, 300.0, -300.0)
            ]
        }
    }
}

impl Placeholder {
    pub fn new (
        placeholder_type: PlaceholderType, 
        valid_spawn: bool, 
        screenspace_visual_rects: Vec<Rect>, 
        worldspace_visual_rects: Vec<Rect>
    ) -> Self {
        Placeholder {placeholder_type, valid_spawn, screenspace_visual_rects, worldspace_visual_rects}
    }

    /// Converts all Rects in the Placeholder's worldspace_visual_rects vector into screenspace.
    /// These are saved, in order, to the Placeholder's screenspace_visual_rects.
    pub fn world_to_screen(&mut self, window_width: f32, window_height: f32) {
        self.screenspace_visual_rects = self.worldspace_visual_rects
            .iter()
            .map(|world_rect| {
                Rect::from_center_size(
                    Vec2::new(
                        (window_width / 2.0) + world_rect.center().x,
                        (window_height / 2.0) - world_rect.center().y
                    ),
                    world_rect.size()
                )
            })
            .collect();
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