use bevy::prelude::*;

use crate::resources_ui::TerritorySettings;

/// `Component`  
/// \
/// 
/// Identifies entity as a `Territory` ui element. A `Territory` can be moved and resized, but cannot overlap with other `Territory`s.
/// \
/// \
/// `Territory`s define a space in which `Tab`s are organized and display their content.
#[derive(Component)]
pub struct Territory {
    pub screenspace_rect: Rect,
    pub worldspace_rect: Rect,
    pub relative_screenspace_rect: Rect,
    pub relative_worldspace_rect: Rect
}
impl Default for Territory {
    fn default() -> Self {
        Territory {
            screenspace_rect: Rect::new(0.0, 0.0, 100.0, 100.0), 
            worldspace_rect: Rect::new(-50.0, -50.0, 50.0, 50.0),
            relative_screenspace_rect: Rect::new(0.0, 0.0, 0.1, 0.1),
            relative_worldspace_rect: Rect::new(-0.05, -0.05, 0.05, 0.05)
        }
    }
}
impl Territory {
    pub fn new(
        screenspace_rect: Rect, 
        worldspace_rect: Rect, 
        relative_screenspace_rect: Rect,
        relative_worldspace_rect: Rect
    ) -> Self {
            Territory {screenspace_rect, worldspace_rect, relative_screenspace_rect, relative_worldspace_rect}
        }

    /// Gets the **screenspace** `Rect` describing the `Territory`'s location in the `Window`.
    pub fn screenspace_rect(&self) -> Rect {
        self.screenspace_rect
    }

    /// Gets the **worldspace** `Rect` describing the `Territory`'s location in the `Window`.
    pub fn worldspace_rect(&self) -> Rect {
        self.worldspace_rect
    }

    /// Gets the relative **screenspace** `Rect` describing the `Territory`'s location in the `Window`.  
    /// \
    /// This `Rect` goes from `0.0` to `1.0` relative to the total size of the `Parent` `Window`.
    pub fn relative_screenspace_rect(&self) -> Rect {
        self.relative_screenspace_rect
    }
    
    /// Gets the relative **worldspace** `Rect` describing the `Territory`'s location in the `Window`.  
    /// \
    /// This `Rect` goes from `-0.5` to `0.5` relative to the total size of the `Parent` `Window`.
    pub fn relative_worldspace_rect(&self) -> Rect {
        self.relative_worldspace_rect
    }

    /// Set a new **screenspace** `Rect`. Requires the `Parent` `Window` dimensions for translation.  
    /// \
    /// **Screenspace** coordinates have the origin `(0.0, 0.0)` in the `Window`'s upper left corner, 
    /// with positive x going right and positive y going down.
    /// - This new **screenspace** `Rect` will be automatically translated to the other coordinate system `Rect`s using:
    ///   - `.screen_to_world()`
    ///   - `.screen_to_relative()`
    ///   - `.world_to_relative()`
    pub fn set_screenspace_rect(&mut self, new_rect: Rect, window_width: f32, window_height: f32) -> &mut Self {
        self.screenspace_rect = new_rect;
        self
            .screen_to_world(window_width, window_height)
            .screen_to_relative(window_width, window_height)
            .world_to_relative(window_width, window_height);
        self
    }

    /// Set a new **worldspace** `Rect`. Requires the `Parent` `Window` dimensions for translation.  
    /// \
    /// **Worldspace** coordinates have the origin `(0.0, 0.0)` in the `Window`'s center, 
    /// with positive x going right and positive y going up.
    /// - This new **worldspace** `Rect` will be automatically translated to the other coordinate system `Rect`s using:
    ///   - `.world_to_screen()`
    ///   - `.world_to_relative()`
    ///   - `.screen_to_relative()`
    pub fn set_worldspace_rect(&mut self, new_rect: Rect, window_width: f32, window_height: f32) -> &mut Self {
        self.worldspace_rect = new_rect;
        self
            .world_to_screen(window_width, window_height)
            .world_to_relative(window_width, window_height)
            .screen_to_relative(window_width, window_height);
        self
    }

    /// Set a new **screenspace** `Rect` in relative coordinates, from `0.0` to `1.0`.
    /// Requires the `Parent` `Window` dimensions for translation.  
    /// \
    /// - This new relative **screenspace** `Rect` will be automatically translated to the other coordinate system `Rect`s using:
    ///   - `.relative_to_screen()`
    ///   - `.screen_to_world()`
    ///   - `.world_to_relative()`
    pub fn set_relative_screenspace_rect(&mut self, new_rect: Rect, window_width: f32, window_height: f32) -> &mut Self {
        self.relative_screenspace_rect = new_rect;
        self
            .relative_to_screen(window_width, window_height)
            .screen_to_world(window_width, window_height)
            .world_to_relative(window_width, window_height);
        self
    }

    /// Set a new **worldspace** `Rect` in relative coordinates, from `-0.5` to `0.5`.
    /// Requires the `Parent` `Window` dimensions for translation.  
    /// \
    /// - This new relative **worldspace** `Rect` will be automatically translated to the other coordinate system `Rect`s using:
    ///   - `.relative_to_world()`
    ///   - `.world_to_screen()`
    ///   - `.screen_to_relative()`
    pub fn set_relative_worldspace_rect(&mut self, new_rect: Rect, window_width: f32, window_height: f32) -> &mut Self {
        self.relative_worldspace_rect = new_rect;
        self
            .relative_to_world(window_width, window_height)
            .world_to_screen(window_width, window_height)
            .screen_to_relative(window_width, window_height);
        self
    }

    /// Moves `Territory.worldspace_rect().center()` some `delta_x` and `delta_y` in **worldspace** coordinates.
    /// Requires the `Parent` `Window` dimensions for translation.  
    /// \
    /// - This new **worldspace** `Rect` will be automatically translated to the other coordinate system `Rect`s using:
    ///   - `.world_to_screen()`
    ///   - `.world_to_relative()`
    ///   - `.screen_to_relative()`
    pub fn move_worldspace_pos(&mut self, delta_x: f32, delta_y: f32, window_width: f32, window_height: f32) -> &mut Self {
        self.worldspace_rect = Rect::from_center_size(
            Vec2::new(
                self.worldspace_rect.center().x + delta_x, 
                self.worldspace_rect.center().y + delta_y
            ), 
            self.worldspace_rect.size()
        );
        self
            .world_to_screen(window_width, window_height)
            .world_to_relative(window_width, window_height)
            .screen_to_relative(window_width, window_height);
        self
    }

    /// Moves `Territory.screenspace_rect().min()` some `delta_x` and `delta_y` in **screenspace** coordinates.
    /// Requires the `Parent` `Window` dimensions for translation.  
    /// \
    /// - This new **screenspace** `Rect` will be automatically translated to the other coordinate system `Rect`s using:
    ///   - `.screen_to_world()`
    ///   - `.screen_to_relative()`
    ///   - `.world_to_relative()`
    pub fn move_screenspace_pos(&mut self, delta_x: f32, delta_y: f32, window_width: f32, window_height: f32) -> &mut Self {
        self.screenspace_rect = Rect::from_corners(
            Vec2::new(
                self.screenspace_rect.min.x + delta_x, 
                self.screenspace_rect.min.y + delta_y
            ), 
            self.screenspace_rect.max
        );
        self
            .screen_to_world(window_width, window_height)
            .screen_to_relative(window_width, window_height)
            .world_to_relative(window_width, window_height);
        self
    }

    /// Updates the `Territory`'s `screenspace_rect` in **screenspace** coordinates to match 
    /// the current `worldspace_rect` in **worldspace** coordinates.  
    /// \
    /// Requires the `Parent` `Window`'s dimensions.
    pub fn world_to_screen(&mut self, window_width: f32, window_height: f32) -> &mut Self {
        self.screenspace_rect = Rect::from_center_size(
            Vec2::new(
            (window_width / 2.0) + self.worldspace_rect.center().x,
            (window_height / 2.0) - self.worldspace_rect.center().y
            ),
            self.worldspace_rect.size()
        );
        self
    }

    /// Updates the `Territory`'s `relative_worldspace_rect` in relative coordinates to match 
    /// the current `worldspace_rect` in **worldspace** coordinates. 
    /// Relative **worldspace** coordinates go from `-0.5` to `0.5` relative to the total size of the `Parent` `Window`.  
    /// \
    /// Requires the `Parent` `Window`'s dimensions.
    pub fn world_to_relative(&mut self, window_width: f32, window_height: f32) -> &mut Self {
        self.relative_worldspace_rect = Rect::new(
            self.worldspace_rect.min.x / window_width, 
            self.worldspace_rect.min.y / window_height, 
            self.worldspace_rect.max.x / window_width, 
            self.worldspace_rect.max.y / window_height
        );
        self
    }

    /// Updates the `Territory`'s `worldspace_rect` in **worldspace** coordinates to match 
    /// the current `screenspace_rect` in **screenspace** coordinates.  
    /// \
    /// Requires the `Parent` `Window`'s dimensions.
    pub fn screen_to_world(&mut self, window_width: f32, window_height: f32) -> &mut Self {
        self.worldspace_rect = Rect::from_center_size(
            Vec2::new(
            self.screenspace_rect.center().x - (window_width / 2.0),
            (window_height / 2.0) - self.screenspace_rect.center().y
            ),
            self.screenspace_rect.size()
        );
        self
    }

    /// Updates the `Territory`'s `relative_screenspace_rect` in relative coordinates to match 
    /// the current `screenspace_rect` in **screenspace** coordinates. 
    /// Relative **screenspace** coordinates go from `0.0` to `1.0` relative to the total size of the `Parent` `Window`.  
    /// \
    /// Requires the `Parent` `Window`'s dimensions.
    pub fn screen_to_relative(&mut self, window_width: f32, window_height: f32) -> &mut Self {
        self.relative_screenspace_rect = Rect::new(
            self.screenspace_rect.min.x / window_width, 
            self.screenspace_rect.min.y / window_height, 
            self.screenspace_rect.max.x / window_width, 
            self.screenspace_rect.max.y / window_height
        );
        self
    }

    /// Updates the `Territory`'s `worldspace_rect` in **worldspace** coordinates to match 
    /// the current `relative_worldspace_rect` in relative coordinates. 
    /// Relative **worldspace** coordinates go from `-0.5` to `0.5` relative to the total size of the `Parent` `Window`.  
    /// \
    /// Requires the `Parent` `Window`'s dimensions.
    pub fn relative_to_world(&mut self, window_width: f32, window_height: f32) -> &mut Self {
        self.worldspace_rect = Rect::new(
            self.relative_worldspace_rect.min.x * window_width, 
            self.relative_worldspace_rect.min.y * window_height,
            self.relative_worldspace_rect.max.x * window_width, 
            self.relative_worldspace_rect.max.y * window_height
        );
        self
    }

    /// Updates the `Territory`'s `screenspace_rect` in **screenspace** coordinates to match 
    /// the current `relative_screenspace_rect` in relative coordinates. 
    /// Relative **screenspace** coordinates go from `0.0` to `1.0` relative to the total size of the `Parent` `Window`.  
    /// \
    /// Requires the `Parent` `Window`'s dimensions.
    pub fn relative_to_screen(&mut self, window_width: f32, window_height: f32) -> &mut Self {
        self.screenspace_rect = Rect::new(
            self.relative_screenspace_rect.min.x / window_width, 
            self.relative_screenspace_rect.min.y / window_height, 
            self.relative_screenspace_rect.max.x / window_width, 
            self.relative_screenspace_rect.max.y / window_height
        );
        self
    }

    /// Checks to see if the `Territory`'s `worldspace_rect` is inside a window's **worldspace** `Rect`.  
    /// \
    /// Be sure to pass in the dimensions of the `Territory`'s `Parent` `Window` and not some other window.
    pub fn is_inside_worldspace_window(&self, window_width: f32, window_height: f32) -> bool {

        let window_rect = Rect::from_center_size(
            Vec2::ZERO, 
            Vec2::new(window_width,window_height)
        );

        window_rect.contains(self.worldspace_rect().min) && window_rect.contains(self.worldspace_rect().max)
    }

    /// Checks to see if the `Territory`'s `screenspace_rect` is inside a window's **screenspace** `Rect`.  
    /// \
    /// Be sure to pass in the dimensions of the `Territory`'s `Parent` `Window` and not some other window.
    pub fn is_inside_screenspace_window(&self, window_width: f32, window_height: f32) -> bool {

        let window_rect = Rect::from_corners(
            Vec2::ZERO, 
            Vec2::new(window_width,window_height)
        );

        window_rect.contains(self.screenspace_rect().min) && window_rect.contains(self.screenspace_rect().max)
    }




    /*




    /// **Worldspace** collision logic adjusting the calling `Territory`'s `worldspace_rect.center()` to move it out of
    /// an `other_territory_rect`. `Rect` size is preserved. Does nothing if there's no intersection between `Rect`s.
    /// Will not move the `Territory` if the collision logic fails to 
    /// \
    /// Don't forget to call `world_to_screen` afterward to translate this to the `screenspace_rect`! 
    pub fn apply_worldspace_drag_collision_with(&mut self, other_territory_rect: Rect) -> &mut Self{
        let conflict_rect = self.worldspace_rect.intersect(other_territory_rect);
        if conflict_rect.is_empty() {return self}
    
        if conflict_rect.height() >= conflict_rect.width() {
            if self.worldspace_rect.center().x >= other_territory_rect.center().x {
                // If the user goes nuts, they can drag Territories fast enough that the conflict rect
                // is entirely contained inside our Territory rect. Remaining space handles that.
                let remaining_space = other_territory_rect.max.x - conflict_rect.max.x;
                self.move_worldspace_pos(conflict_rect.width() + remaining_space, 0.0);
            }
            else {
                let remaining_space = conflict_rect.min.x - other_territory_rect.min.x;
                self.move_worldspace_pos(-1.0 * conflict_rect.width() - remaining_space, 0.0);
            }
        }
        else {
            if self.worldspace_rect.center().y >= other_territory_rect.center().y {
                let remaining_space = other_territory_rect.max.y - conflict_rect.max.y;
                self.move_worldspace_pos(0.0, conflict_rect.height() + remaining_space);
            }
            else {
                let remaining_space = conflict_rect.min.y - other_territory_rect.min.y;
                self.move_worldspace_pos(0.0, -1.0 * conflict_rect.height() - remaining_space);
            } 
        }

        // If we still have a conflict rect, reject the move entirely.

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





    */

}

/// Marks a `Territory Tabs` UI element as having been commanded to move. Entities with this component will be processed 
/// by motion systems and this component will be removed once all processing is complete.
#[derive(Component)]
pub struct MoveRequested {
    pub proposed_screenspace_rect: Rect,
    pub proposed_worldspace_rect: Rect
}
impl MoveRequested {
    pub fn new (proposed_screenspace_rect: Rect, proposed_worldspace_rect: Rect) -> Self {
        MoveRequested {proposed_screenspace_rect, proposed_worldspace_rect}
    }

    /// Builds from the moving UI element's `screenspace_rect` and a **screenspace** `delta_min` & `delta_max`.
    /// Translates this into the `proposed_worldspace_rect` as well.
    pub fn from_screenspace_delta (
        &mut self,
        screenspace_rect: Rect, 
        delta_min: Vec2, 
        delta_max: Vec2, 
        window_width: f32, 
        window_height: f32
    ) -> &mut Self {
        let mut constructed = MoveRequested { 
            proposed_screenspace_rect: Rect::from_corners(
                screenspace_rect.min + delta_min, 
                screenspace_rect.max + delta_max
            ), 
            proposed_worldspace_rect: Rect::from_corners(Vec2::ZERO, Vec2::ONE)
        };
        constructed.screen_to_world(window_width, window_height)
    }

    /// Builds from the moving UI element's `worldspace_rect` and a **worldspace** `delta_min` & `delta_max`.
    /// Translates this into the `proposed_screenspace_rect` as well.
    pub fn from_worldspace_delta (
        &mut self,
        worldspace_rect: Rect, 
        delta_min: Vec2, 
        delta_max: Vec2, 
        window_width: f32, 
        window_height: f32
    ) -> &mut Self {
        let mut constructed = MoveRequested { 
            proposed_screenspace_rect: Rect::from_corners(Vec2::ZERO, Vec2::ONE), 
            proposed_worldspace_rect: Rect::from_corners(
                worldspace_rect.min + delta_min, 
                worldspace_rect.max + delta_max
            ) 
        };
        constructed.world_to_screen(window_width, window_height)
    }

    /// Getter for the **screenspace** `Rect` that the UI element wants to move to.
    pub fn proposed_screenspace_rect (&self) -> Rect {self.proposed_screenspace_rect}

    /// Getter for the **worldspace** `Rect` that the UI element wants to move to.
    pub fn proposed_worldspace_rect (&self) -> Rect {self.proposed_worldspace_rect}

    /// Changes the **screenspace** `Rect` that the UI element wants to move to. Automatic translation to other `Rect`.
    pub fn set_proposed_screenspace_rect (&mut self, new_rect: Rect, window_width: f32, window_height: f32) -> &mut Self {
        self.proposed_screenspace_rect = new_rect;
        self.screen_to_world(window_width, window_height);
        self
    }

    /// Changes the **worldspace** `Rect` that the UI element wants to move to. Automatic translation to other `Rect`.
    pub fn set_proposed_worldspace_rect (&mut self, new_rect: Rect, window_width: f32, window_height: f32) -> &mut Self {
        self.proposed_worldspace_rect = new_rect;
        self.world_to_screen(window_width, window_height);
        self
    }

    /// Updates the `proposed_screenspace_rect` in **screenspace** coordinates to match 
    /// the current `proposed_worldspace_rect` in **worldspace** coordinates.  
    /// \
    /// Requires the parent `Window`'s dimensions.
    pub fn world_to_screen(&mut self, window_width: f32, window_height: f32) -> &mut Self{
        self.set_proposed_screenspace_rect(
            Rect::from_center_size(
                Vec2::new(
                (window_width / 2.0) + self.proposed_worldspace_rect().center().x,
                (window_height / 2.0) - self.proposed_worldspace_rect().center().y
                ),
                self.proposed_worldspace_rect().size(),
            ),
            window_width,
            window_height
        )
    }

    /// Updates the `proposed_worldspace_rect` in **worldspace** coordinates to match 
    /// the current `proposed_screenspace_rect` in **screenspace** coordinates.  
    /// \
    /// Requires the parent `Window`'s dimensions.
    pub fn screen_to_world(&mut self, window_width: f32, window_height: f32) -> &mut Self {
        self.set_proposed_worldspace_rect(
            Rect::from_center_size(
                Vec2::new(
                self.proposed_screenspace_rect().center().x - (window_width / 2.0),
                (window_height / 2.0) - self.proposed_screenspace_rect().center().y
                ),
                self.proposed_screenspace_rect().size()
            ),
            window_width,
            window_height
        )
    }

    /// Moves the `proposed_screenspace_rect` some `delta_min` & `delta_max`. Automatic translation to other `Rect`.
    pub fn move_screenspace_delta (&mut self, delta_min: Vec2, delta_max: Vec2, window_width: f32, window_height: f32) -> &mut Self {
        self
            .set_proposed_screenspace_rect(
                Rect::from_corners(
                    self.proposed_screenspace_rect().min + delta_min, 
                    self.proposed_screenspace_rect().max + delta_max
                ),
                window_width,
                window_height
            )
            .screen_to_world(window_width, window_height)
    }

    /// Moves the `proposed_worldspace_rect` some `delta_min` & `delta_max`. Automatic translation to other `Rect`.
    pub fn move_worldspace_delta (&mut self, delta_min: Vec2, delta_max: Vec2, window_width: f32, window_height: f32) -> &mut Self {
        self
            .set_proposed_worldspace_rect(
                Rect::from_corners(
                    self.proposed_worldspace_rect().min + delta_min, 
                    self.proposed_worldspace_rect().max + delta_max
                ),
                window_width,
                window_height
            )
            .world_to_screen(window_width, window_height)
    }

}

/// `Component`  
/// \
/// User has marked this UI element as `Locked`, and they don't want any systems moving it around!
#[derive(Component)]
pub struct Locked;

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

/// `Component`  
/// \
/// Combined with a `Window` component, denotes a window entity as the place to run `Territory Tabs` logic.
#[derive(Component)]
pub struct TerritoryTabs;

/// `Component`  
/// \
/// Defines what library will be used to display UI. Add to a `Window` entity to set a default. Add to a `Territory`
/// or a `Tab` entity to override that default.
#[derive(Component)]
pub enum DisplayLibrary {
    Default,
    BevyUi,
    BevyEgui,
    BevySickle
}

/// This marks a camera as being intended for use as a 2D world UI background camera.
/// Mouse seeking systems will check cameras with this component.
#[derive(Component)]
pub struct MouseSeekingCamera;
