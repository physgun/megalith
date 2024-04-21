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
    pub screenspace_rect: Rect,
    pub worldspace_rect: Rect,
    pub orientation: Orientation,
    pub flipped: bool,
    pub locked: bool
}
impl Default for Territory {
    fn default() -> Self {
        Territory {
            screenspace_rect: Rect::new(0.0, 0.0, 100.0, 100.0), 
            worldspace_rect: Rect::new(0.0, 0.0, 100.0, 100.0), 
            orientation: Orientation::Vertical,
            flipped: false,
            locked: false
        }
    }
}
impl Territory {
    pub fn new(screenspace_rect: Rect, worldspace_rect: Rect, orientation: Orientation, flipped: bool, locked: bool) -> Self {
        Territory {screenspace_rect, worldspace_rect, orientation, flipped, locked}
    }

    /// Moves `Territory.worldspace_rect.center()` some `delta_x` and `delta_y` in **worldspace** coordinates.  
    /// \
    /// Don't forget to call `world_to_screen` afterward to translate this to the `Territory.screenspace_rect`! 
    pub fn move_worldspace_pos(&mut self, delta_x: f32, delta_y: f32) {
        self.worldspace_rect = Rect::from_center_size(
            Vec2::new(
                self.worldspace_rect.center().x + delta_x, 
                self.worldspace_rect.center().y + delta_y
            ), 
            self.worldspace_rect.size()
        );
    }

    /// Take the `Territory`'s current **worldspace** `Rect` and converts into a **screenspace** `Rect`. 
    /// The **screenspace** `Rect` is not returned but instead applied to the calling `Territory`.  
    /// \
    /// Requires the parent `Window`'s **screenspace** dimensions.
    pub fn world_to_screen(&mut self, window_width: f32, window_height: f32) {
        self.screenspace_rect = Rect::from_center_size(
            Vec2::new(
            (window_width / 2.0) + self.worldspace_rect.center().x,
            (window_height / 2.0) - self.worldspace_rect.center().y
            ),
            self.worldspace_rect.size()
        );
    }

    /// Take the `Territory`'s current **screenspace** `Rect` and converts into a **worldspace** `Rect`. 
    /// The **worldspace** `Rect` is not returned but instead applied to the calling `Territory`.  
    /// \
    /// Requires the parent `Window`'s **screenspace** dimensions.
    pub fn screen_to_world(&mut self, window_width: f32, window_height: f32) {
        self.worldspace_rect = Rect::from_center_size(
            Vec2::new(
            self.screenspace_rect.center().x - (window_width / 2.0),
            (window_height / 2.0) - self.screenspace_rect.center().y
            ),
            self.screenspace_rect.size()
        );
    }

    /// **Worldspace** collision logic adjusting the calling `Territory`'s `worldspace_rect.center()` to move it out of
    /// an `other_territory_rect`. Does nothing if there's no intersection between `Rect`s.  
    /// \
    /// Don't forget to call `world_to_screen` afterward to translate this to the `Territory.screenspace_rect`! 
    pub fn world_drag_collision(&mut self, other_territory_rect: Rect) {
        let conflict_rect = self.worldspace_rect.intersect(other_territory_rect);
        
        if conflict_rect.is_empty() {return}

        if conflict_rect.height() >= conflict_rect.width() {
            if self.worldspace_rect.center().x >= other_territory_rect.center().x {
                self.move_worldspace_pos(conflict_rect.width(), 0.0);
            }
            else {
                self.move_worldspace_pos(-conflict_rect.width(), 0.0);
            }
        }
        else {
            if self.worldspace_rect.center().y >= other_territory_rect.center().y {
                self.move_worldspace_pos(0.0, conflict_rect.height());
            }
            else {
                self.move_worldspace_pos(0.0, -conflict_rect.height());
            } 
        }
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
