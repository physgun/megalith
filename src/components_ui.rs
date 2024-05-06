use bevy::prelude::*;

use crate::resources_ui::TerritorySettings;

// Territories and Tabs can have a horizontal or vertical orientation.
#[derive(Copy, Clone, Default)]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal
}

// Identifies entity as a Territory that contains Tabs. Typically needs Tabs, and will be cleaned up if it has none.
// Territories will manage Tabs through Parent / Child components.
#[derive(Component)]
pub struct Territory {
    screenspace_rect: Rect,
    worldspace_rect: Rect,
    orientation: Orientation,
    flipped: bool,
    locked: bool
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
    pub fn new(
        screenspace_rect: Rect, 
        worldspace_rect: Rect, 
        orientation: Orientation, 
        flipped: bool, 
        locked: bool) -> Self {
        Territory {screenspace_rect, worldspace_rect, orientation, flipped, locked}
    }

    /// Gets the **screenspace** `Rect` describing the `Territory`'s location in the `Window`.
    pub fn screenspace_rect(&self) -> Rect {
        self.screenspace_rect
    }

    /// Gets the **worldspace** `Rect` describing the `Territory`'s location in the `Window`.
    pub fn worldspace_rect(&self) -> Rect {
        self.worldspace_rect
    }

    /// Gets the `Tab` `Orientation` of the `Territory`. `Vertical` has `Tab`s on top or bottom, 
    /// `Horizontal` has `Tab`s on left or right.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Returns `true` is the `Territory` has been locked. Many operations modifying `Territory`s won't
    /// modify a locked `Territory`.
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// Returns `true` if a `Territory` has been flipped. Flipping a `Territory` places the `Tab`s on the opposite
    /// side of the `Territory` per it's `Orientation`. Default is for `Tab`s to be on top or on the left when `false`.
    pub fn is_flipped(&self) -> bool {
        self.flipped
    }

    /// Set a new **screenspace** `Rect`.  
    /// \
    /// Don't forget to call `screen_to_world` to update the `Territory`s `worldspace_rect`.
    pub fn set_screenspace_rect(&mut self, new_rect: Rect) -> &mut Self {
        self.screenspace_rect = new_rect;
        self
    }

    /// Set a new **worldspace** `Rect`.  
    /// \
    /// Don't forget to call `world_to_screen` to update the `Territory`s `screenspace_rect`.
    pub fn set_worldspace_rect(&mut self, new_rect: Rect) -> &mut Self {
        self.worldspace_rect = new_rect;
        self
    }

    /// Change the `Territory` `Orientation`. `Orientation::Vertical` by default.
    pub fn set_orientation(&mut self, new_orientation: Orientation) -> &mut Self {
        self.orientation = new_orientation;
        self
    }

    /// Flip the `Territory` such that the `Tab`s are on the other side.
    pub fn flip(&mut self) -> &mut Self {
        self.flipped = !self.flipped;
        self
    }

    /// Lock the `Territory`, preventing modification or movement from most methods.
    pub fn lock(&mut self) -> &mut Self {
        self.locked = true;
        self
    }

    /// Unlocks the `Territory`, exposing it to modification or movement.
    pub fn unlock(&mut self) -> &mut Self {
        self.locked = false;
        self
    }

    /// Moves `Territory.worldspace_rect().center()` some `delta_x` and `delta_y` in **worldspace** coordinates.  
    /// \
    /// Don't forget to call `world_to_screen` afterward to translate this to the `Territory`'s `screenspace_rect`! 
    pub fn move_worldspace_pos(&mut self, delta_x: f32, delta_y: f32) -> &mut Self {
        self.worldspace_rect = Rect::from_center_size(
            Vec2::new(
                self.worldspace_rect.center().x + delta_x, 
                self.worldspace_rect.center().y + delta_y
            ), 
            self.worldspace_rect.size()
        );
        self
    }

    /// Moves `Territory.screenspace_rect().min()` some `delta_x` and `delta_y` in **screenspace** coordinates.  
    /// \
    /// Don't forget to call `screen_to_world` afterward to translate this to the `Territory`'s `worldspace_rect`! 
    pub fn move_screenspace_pos(&mut self, delta_x: f32, delta_y: f32) -> &mut Self {
        self.screenspace_rect = Rect::from_corners(
            Vec2::new(
                self.screenspace_rect.min.x + delta_x, 
                self.screenspace_rect.min.y + delta_y
            ), 
            self.screenspace_rect.max
        );
        self
    }

    /// Updates the `Territory`'s `screenspace_rect` in **screenspace** coordinates to match 
    /// the current `worldspace_rect` in **worldspace** coordinates.  
    /// \
    /// Requires the parent `Window`'s **screenspace** dimensions.
    pub fn world_to_screen(&mut self, window_width: f32, window_height: f32) -> &mut Self{
        self.screenspace_rect = Rect::from_center_size(
            Vec2::new(
            (window_width / 2.0) + self.worldspace_rect.center().x,
            (window_height / 2.0) - self.worldspace_rect.center().y
            ),
            self.worldspace_rect.size()
        );
        self
    }

    /// Updates the `Territory`'s `worldspace_rect` in **worldspace** coordinates to match 
    /// the current `screenspace_rect` in **screenspace** coordinates.  
    /// \
    /// Requires the parent `Window`'s **screenspace** dimensions.
    pub fn screen_to_world(&mut self, window_width: f32, window_height: f32) -> &mut Self{
        self.worldspace_rect = Rect::from_center_size(
            Vec2::new(
            self.screenspace_rect.center().x - (window_width / 2.0),
            (window_height / 2.0) - self.screenspace_rect.center().y
            ),
            self.screenspace_rect.size()
        );
        self
    }

    /// **Worldspace** collision logic adjusting the calling `Territory`'s `worldspace_rect.center()` to move it out of
    /// an `other_territory_rect`. `Rect` size is preserved. Does nothing if there's no intersection between `Rect`s.  
    /// \
    /// Don't forget to call `world_to_screen` afterward to translate this to the `Territory.screenspace_rect`! 
    pub fn apply_worldspace_drag_collision_with(&mut self, other_territory_rect: Rect) -> &mut Self{
        let conflict_rect = self.worldspace_rect.intersect(other_territory_rect);
        
        if conflict_rect.is_empty() {return self}

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
        self
    }

    /// Checks if the `Territory` can be resized by some `delta_min` and / or `delta_max` in **worldspace** coordinates.  
    /// If you're passing in **screenspace** deltas, flip the sign of the `.y` components for both deltas.  
    /// Will return `false` if the `Territory` is locked.
    pub fn can_resize_by(&self, delta_min: Vec2, delta_max: Vec2) -> bool{
        if self.locked {return false};
        if self.worldspace_rect.width() - delta_min.x + delta_max.x < TerritorySettings::default().min_size.x {return false};
        if self.worldspace_rect.height() - delta_min.y + delta_max.y < TerritorySettings::default().min_size.y {return false};
        true
    }

    /// Resizes Territory by its `.min` and `max` points by some `delta_min` and / or `delta_max` in **worldspace** coordinates.  
    /// If you're passing in **screenspace** deltas, flip the sign of the `.y` components for both deltas.  
    /// Does not care about minimum sizes and will resize to whatever commanded. 
    /// Call `Territory.can_resize_by()` to determine if constraints will be violated before calling this method.
    /// Does nothing if the `Territory` is locked.  
    /// \
    /// Don't forget to call `world_to_screen` to transfer the new size to the `screenspace_rect`.
    pub fn resize_by(&mut self, delta_min: Vec2, delta_max: Vec2) -> &mut Self{
        if self.locked {warn!("[TERRITORY] .resize_by() called for a locked Territory!"); return self;}
        self.worldspace_rect.min += delta_min;
        self.worldspace_rect.max += delta_max;
        self
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
