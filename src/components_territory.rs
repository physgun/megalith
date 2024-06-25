//! Contains all States, Resources, and Components pertaining to a [`Territory`].

use bevy::prelude::*;

/// Smallest size of an icon.
const ICON_SIZE: Vec2 = Vec2 { x: 20.0, y: 20.0 };

/// Settings governing the basic size behavior of all entities with [`Territory`] components. 
#[derive(Resource)]
pub struct GlobalTerritorySettings {
    /// Smallest possible size of a [`Territory`]. Defaults to the size of a single icon.
    pub min_size: Vec2,
    /// Starting size when spawning a new [`Territory`].
    pub default_size: Vec2,
    /// Distance of the tabs from the frame of the [`Territory`].
    pub inner_margins: Vec2,
    /// Distance of everything outside from the frame of the [`Territory`]. This will govern the space between them.
    pub outer_margins: Vec2
}
impl Default for GlobalTerritorySettings{
    fn default() -> Self {
        GlobalTerritorySettings {
            min_size: ICON_SIZE,
            default_size: Vec2 { x: 600.0, y: 200.0 },
            inner_margins: Vec2 { x: 3.0, y: 3.0 },
            outer_margins: Vec2 { x: 2.5, y: 2.5 }
        }
    }
}

/// Combined with a `Window` component, denotes a window entity as a space to run `Territory Tabs` logic.
/// Display libraries will attach their root nodes and contexts to the entity with this component.
#[derive(Component)]
pub struct TerritoryTabs;

/// Identifies the camera that will display `Territory Tabs` UI.
#[derive(Component)]
pub struct TerritoryTabsCamera;

#[derive(Component)]
/// Identifies the UI Root Node associated with a [`Window`] [`Entity`].
pub struct TerritoryTabsUIRoot {
    /// The [`Window`] [`Entity`] this root node marker component is associated with, but not attached directly to.
    /// This is a different [`Entity`] than the one the root node bundle & [`TerritoryTabsUIRoot`] is attached to!  
    /// \
    /// bevy_ui queries for root nodes by looking for nodes without a [`Parent`], so the root node can't be connected
    /// to the [`Window`] that way. Another use case for entity relations, when they get here!
    pub associated_window_entity: Entity
}

/// Denotes the [`Entity`] as containing the base node for a [`Territory`] [`Entity`].
#[derive(Component)]
pub struct TerritoryBaseNode;

/// Denotes the [`Entity`] as containing the drag node for a [`Territory`] [`Entity`].
#[derive(Component)]
pub struct TerritoryDragNode;

/// Denotes the [`Entity`] as containing the resize grid node for a [`Territory`] [`Entity`].
#[derive(Component)]
pub struct TerritoryResizeGridNode;

/// Denotes the [`Entity`] as containing the individual resize button node for a [`Territory`] [`Entity`].
#[derive(Component)]
pub struct TerritoryResizeButtonNode;


/// App State communicating the operating Mode of the `Territory Tabs` UI.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TerritoryTabsMode {
    #[default]
    /// The user has managed to remove everything, leaving them stuck. A button to spawn a Territory should be presented.
    Empty,
    /// Nominal state. The user is operating features present in the UI.
    Operating,
    /// User is changing the layout. Helper overlays should be spawned. 
    MovingTerritories,
    /// User is repositioning a feature, and may spawn a new Territory.
    MovingTabs
}

/// User has marked this UI element as `Locked`, and they don't want any systems moving it around!
#[derive(Component)]
pub struct Locked;

/// Defines what library will be used to display UI. Add to a `Window` entity to set a default. Add to a `Territory`
/// or a `Tab` entity to override that default.
#[derive(Component, Clone, Copy)]
pub enum DisplayLibrary {
    BevyUi,
    BevyEgui,
    BevySickle
}

/// Every UI library that handles resizing has this exact enum. This idea with having our own here 
/// is to implement an extension trait for translating to each library, but only in the modules that interact 
/// with that library. Hopefully this will maintain both a decoupled architecture with the 
/// display libraries and to keep Territory Tabs flexible with regard to what libraries it can use.
#[derive(Component, Clone, Copy, Debug)]
pub enum ResizeDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest
}

impl ResizeDirection {
    
    /// Width of the resizing bar buttons, and both the height and width of the corner ones.
    pub const SIZE: f32 = 5.0;

    /// Helper for iterating through all the directions.
    pub const ALL: [Self; 8] = [
        Self::North,
        Self::NorthEast,
        Self::East,
        Self::SouthEast,
        Self::South,
        Self::SouthWest,
        Self::West,
        Self::NorthWest
    ];

    /// Gets the opposite resize direction.
    pub fn get_opposite(&self) -> Self {
        match self {
            Self::North => { ResizeDirection::South },
            Self::NorthEast => { ResizeDirection::SouthWest },
            Self::East => { ResizeDirection::West },
            Self::SouthEast => { ResizeDirection::NorthWest },
            Self::South => { ResizeDirection::North },
            Self::SouthWest => { ResizeDirection::NorthEast },
            Self::West => { ResizeDirection::East },
            Self::NorthWest => { ResizeDirection::SouthEast }
        }
    }

    /// Gets the adjacent sides.  
    ///   
    /// If the [`ResizeDirection`] is a corner, gets the connecting sides. If a side, then the corners bordering.
    /// Ordering is clockwise.
    pub fn get_both_adjacent(&self) -> (Self, Self) {
        match self {
            Self::North => { (ResizeDirection::NorthWest, ResizeDirection::NorthEast) },
            Self::NorthEast => { (ResizeDirection::North, ResizeDirection::East) },
            Self::East => { (ResizeDirection::NorthEast, ResizeDirection::SouthEast) },
            Self::SouthEast => { (ResizeDirection::East, ResizeDirection::South) },
            Self::South => { (ResizeDirection::SouthEast, ResizeDirection::SouthWest) },
            Self::SouthWest => { (ResizeDirection::South, ResizeDirection::West) },
            Self::West => { (ResizeDirection::SouthWest, ResizeDirection::NorthWest) },
            Self::NorthWest => { (ResizeDirection::West, ResizeDirection::North) }
        }
    }

    /// If you're using a 3x3 CSS grid node to place the resize drag buttons, 
    /// you can use this to get the appropriate (row, column) location.  
    ///   
    /// If called on a [`ResizeDirection::West`], then you'll get `(GridPlacement::start(2), GridPlacement::start(1))`.
    pub fn get_css_grid_location(&self) -> (GridPlacement, GridPlacement) {
        let (row, column) = match self {
            Self::North => { ( 1, 2) },
            Self::NorthEast => { (1, 3) },
            Self::East => { (2, 3 ) },
            Self::SouthEast => { (3, 3) },
            Self::South => { (3, 2) },
            Self::SouthWest => { (3, 1) },
            Self::West => { (2, 1) },
            Self::NorthWest => { (1, 1) }
        };
        (GridPlacement::start(row), GridPlacement::start(column))
    }

    /// Add the correct mouse delta [`Vec2`], depending on [`ResizeDirection`], to a [`Rect`] in **screenspace** coordinates.  
    ///  
    /// If, say, [`ResizeDirection::SouthWest`], then the returned [`Rect`] will be the result of `rect.min.x += delta.x; rect.max.y += delta.y`:
    pub fn add_delta_to_rect(&self, mut rect: Rect, delta: Vec2) -> Rect {
        match self {
            Self::North => { rect.min.y += delta.y; },
            Self::NorthEast => { rect.min.y += delta.y; rect.max.x += delta.x },
            Self::East => { rect.max.x += delta.x },
            Self::SouthEast => { rect.max += delta; },
            Self::South => { rect.max.y += delta.y },
            Self::SouthWest => { rect.min.x += delta.x; rect.max.y += delta.y },
            Self::West => { rect.min.x += delta.x },
            Self::NorthWest => { rect.min += delta },
        }
        rect
    }
}

/// A collection of `Bevy` [`Rect`]s that are useful to a variety of UI libraries.  
/// \
/// 
/// So long as you pass in the correct `Window` dimensions, this component will automatically translate between all [`Rect`]s.
/// Contains helper methods to deal with all of the different coordinate systems.
#[derive(Component, Clone, Copy)]
pub struct RectKit {
    /// Origin at top left of the screen, `+x` goes right and `+y` goes down. `.min()` is top left while `.max()` is bottom right.
    pub screenspace: Rect,
    /// Origin at center of the screen, `+x` goes right and `+y` goes up. `.min()` is bottom left while `.max()` is top right.
    pub worldspace: Rect,
    /// [`RectKit::screenspace`] but with coordinates mapped from `(0.0, 0.0)` at top left to `(1.0, 1.0)` at bottom right.
    pub relative_screenspace: Rect,
    /// [`RectKit::worldspace`] but with coordinates mapped from `(-0.5, -0.5)` at bottom left to `(0.5, 0.5)` at top right.
    pub relative_worldspace: Rect
}
impl Default for RectKit {
    fn default() -> Self {
        RectKit {
            screenspace: Rect::new(0.0, 0.0, 100.0, 100.0), 
            worldspace: Rect::new(-50.0, -50.0, 50.0, 50.0),
            relative_screenspace: Rect::new(0.0, 0.0, 0.1, 0.1),
            relative_worldspace: Rect::new(-0.05, -0.05, 0.05, 0.05)
        }
    }
}
impl RectKit {
    pub fn new(
        screenspace: Rect, 
        worldspace: Rect, 
        relative_screenspace: Rect,
        relative_worldspace: Rect
    ) -> Self {
            RectKit {screenspace, worldspace, relative_screenspace, relative_worldspace}
        }

    /// Creates a [`RectKit`] with all zero-sized [`Rect`]s.
    pub fn empty() -> Self {
        let rect_zero = Rect::from_corners(Vec2::ZERO, Vec2::ZERO);
        RectKit {
            screenspace: rect_zero, 
            worldspace: rect_zero, 
            relative_screenspace: rect_zero, 
            relative_worldspace: rect_zero
        }
    }

    /// Creates a complete [`RectKit`] from a **screenspace** [`Rect`].
    pub fn from_screenspace (new_rect: Rect, window_width: f32, window_height: f32) -> Self {
        *RectKit::empty().set_screenspace(new_rect, window_width, window_height)
    }

    /// Creates a complete [`RectKit`] from a **worldspace** [`Rect`].
    pub fn from_worldspace (new_rect: Rect, window_width: f32, window_height: f32) -> Self {
        *RectKit::empty().set_worldspace(new_rect, window_width, window_height)
    }

    /// Creates a complete [`RectKit`] from a relative **screenspace** [`Rect`].
    pub fn from_relative_screenspace (new_rect: Rect, window_width: f32, window_height: f32) -> Self {
        *RectKit::empty().set_relative_screenspace(new_rect, window_width, window_height)
    }

    /// Creates a complete [`RectKit`] from a relative **worldspace** [`Rect`].
    pub fn from_relative_worldspace (new_rect: Rect, window_width: f32, window_height: f32) -> Self {
        *RectKit::empty().set_relative_worldspace(new_rect, window_width, window_height)
    }

    /// Gets the **screenspace** [`Rect`] describing a location in the `Window`.
    pub fn screenspace(&self) -> Rect {
        self.screenspace
    }

    /// Gets the **worldspace** [`Rect`] describing a location in the `Window`.
    pub fn worldspace(&self) -> Rect {
        self.worldspace
    }

    /// Gets the relative **screenspace** [`Rect`] describing a location in the `Window`.  
    /// \
    /// This [`Rect`] ranges from `0.0` to `1.0` relative to the total size of the `Window`.
    pub fn relative_screenspace(&self) -> Rect {
        self.relative_screenspace
    }
    
    /// Gets the relative **worldspace** [`Rect`] describing a location in the `Window`.  
    /// \
    /// This [`Rect`] ranges from `-0.5` to `0.5` relative to the total size of the `Window`.
    pub fn relative_worldspace(&self) -> Rect {
        self.relative_worldspace
    }

    /// Set a new **screenspace** [`Rect`]. Requires the appropriate `Window` dimensions for translation.  
    /// \
    /// **Screenspace** coordinates have the origin `(0.0, 0.0)` in the `Window`'s upper left corner, 
    /// with positive x going right and positive y going down.
    /// - This new **screenspace** [`Rect`] will be automatically translated to the other coordinate system [`Rect`]s using:
    ///   - [`RectKit::screen_to_world`]
    ///   - [`RectKit::screen_to_relative`]
    ///   - [`RectKit::world_to_relative`]
    pub fn set_screenspace(&mut self, new_rect: Rect, window_width: f32, window_height: f32) -> &mut Self {
        self.screenspace = new_rect;
        self
            .screen_to_world(window_width, window_height)
            .screen_to_relative(window_width, window_height)
            .world_to_relative(window_width, window_height)
    }

    /// Set a new **worldspace** [`Rect`]. Requires the appropriate `Window` dimensions for translation.  
    /// \
    /// **Worldspace** coordinates have the origin `(0.0, 0.0)` in the `Window`'s center, 
    /// with positive x going right and positive y going up.
    /// - This new **worldspace** [`Rect`] will be automatically translated to the other coordinate system [`Rect`]s using:
    ///   - [`RectKit::world_to_screen`]
    ///   - [`RectKit::world_to_relative`]
    ///   - [`RectKit::screen_to_relative`]
    pub fn set_worldspace(&mut self, new_rect: Rect, window_width: f32, window_height: f32) -> &mut Self {
        self.worldspace = new_rect;
        self
            .world_to_screen(window_width, window_height)
            .world_to_relative(window_width, window_height)
            .screen_to_relative(window_width, window_height)
    }

    /// Set a new **screenspace** [`Rect`] in relative coordinates, from `0.0` to `1.0`.
    /// Requires the appropriate `Window` dimensions for translation.  
    /// \
    /// - This new relative **screenspace** [`Rect`] will be automatically translated to the other coordinate system [`Rect`]s using:
    ///   - [`RectKit::relative_to_screen`]
    ///   - [`RectKit::screen_to_world`]
    ///   - [`RectKit::world_to_relative`]
    pub fn set_relative_screenspace(&mut self, new_rect: Rect, window_width: f32, window_height: f32) -> &mut Self {
        self.relative_screenspace = new_rect;
        self
            .relative_to_screen(window_width, window_height)
            .screen_to_world(window_width, window_height)
            .world_to_relative(window_width, window_height)
    }

    /// Set a new **worldspace** [`Rect`] in relative coordinates, from `-0.5` to `0.5`.
    /// Requires the appropriate `Window` dimensions for translation.  
    /// \
    /// - This new relative **worldspace** [`Rect`] will be automatically translated to the other coordinate system [`Rect`]s using:
    ///   - [`RectKit::relative_to_world`]
    ///   - [`RectKit::world_to_screen`]
    ///   - [`RectKit::screen_to_relative`]
    pub fn set_relative_worldspace(&mut self, new_rect: Rect, window_width: f32, window_height: f32) -> &mut Self {
        self.relative_worldspace = new_rect;
        self
            .relative_to_world(window_width, window_height)
            .world_to_screen(window_width, window_height)
            .screen_to_relative(window_width, window_height)
    }

    /// Moves [`RectKit::worldspace`]'s [`Rect::center`] some `delta_x` and `delta_y` in **worldspace** coordinates.
    /// Requires the appropriate `Window` dimensions for translation.  
    /// \
    /// - This new **worldspace** [`Rect`] will be automatically translated to the other coordinate system [`Rect`]s using:
    ///   - [`RectKit::world_to_screen`]
    ///   - [`RectKit::world_to_relative`]
    ///   - [`RectKit::screen_to_relative`]
    pub fn move_worldspace_pos(&mut self, delta_x: f32, delta_y: f32, window_width: f32, window_height: f32) -> &mut Self {
        self.worldspace = Rect::from_center_size(
            Vec2::new(
                self.worldspace.center().x + delta_x, 
                self.worldspace.center().y + delta_y
            ), 
            self.worldspace.size()
        );
        self
            .world_to_screen(window_width, window_height)
            .world_to_relative(window_width, window_height)
            .screen_to_relative(window_width, window_height)
    }

    /// Moves [`RectKit::worldspace`]'s minimum and maximum corners
    /// some `delta_min` and `delta_max` in **worldspace** coordinates. So, bottom left and top right points of the [`Rect`].
    /// Requires the appropriate `Window` dimensions for translation.  
    /// \
    /// - This new **worldspace** [`Rect`] will be automatically translated to the other coordinate system [`Rect`]s using:
    ///   - [`RectKit::world_to_screen`]
    ///   - [`RectKit::world_to_relative`]
    ///   - [`RectKit::screen_to_relative`]
    pub fn move_worldspace_corners(&mut self, delta_min: Vec2, delta_max: Vec2, window_width: f32, window_height: f32) -> &mut Self {
        self.worldspace = Rect::from_corners(
            self.worldspace.min + delta_min,
            self.worldspace.max + delta_max
        );
        self
            .world_to_screen(window_width, window_height)
            .world_to_relative(window_width, window_height)
            .screen_to_relative(window_width, window_height)
    }

    /// Moves [`RectKit::screenspace`]'s [`Rect::min`] some `delta_x` and `delta_y` in **screenspace** coordinates.
    /// Requires the appropriate `Window` dimensions for translation.  
    /// \
    /// - This new **screenspace** [`Rect`] will be automatically translated to the other coordinate system [`Rect`]s using:
    ///   - [`RectKit::screen_to_world`]
    ///   - [`RectKit::screen_to_relative`]
    ///   - [`RectKit::world_to_relative`]
    pub fn move_screenspace_pos(&mut self, delta_x: f32, delta_y: f32, window_width: f32, window_height: f32) -> &mut Self {
        self.screenspace = Rect::from_corners(
            Vec2::new(
                self.screenspace.min.x + delta_x, 
                self.screenspace.min.y + delta_y
            ), 
            Vec2::new(
                self.screenspace.max.x + delta_x, 
                self.screenspace.max.y + delta_y
            )
        );
        self
            .screen_to_world(window_width, window_height)
            .screen_to_relative(window_width, window_height)
            .world_to_relative(window_width, window_height)
    }

    /// Moves [`RectKit::screenspace`]'a minimum and maximum corners
    /// some `delta_min` and `delta_max` in **screenspace** coordinates. So, top left and bottom right points of the [`Rect`].
    /// Requires the appropriate `Window` dimensions for translation.  
    /// \
    /// - This new **screenspace** [`Rect`] will be automatically translated to the other coordinate system [`Rect`]s using:
    ///   - [`RectKit::screen_to_world`]
    ///   - [`RectKit::screen_to_relative`]
    ///   - [`RectKit::world_to_relative`]
    pub fn move_screenspace_corners(&mut self, delta_min: Vec2, delta_max: Vec2, window_width: f32, window_height: f32) -> &mut Self {
        self.screenspace = Rect::from_corners(
            self.screenspace.min + delta_min, 
            self.screenspace.max + delta_max
        );
        self
            .screen_to_world(window_width, window_height)
            .screen_to_relative(window_width, window_height)
            .world_to_relative(window_width, window_height)
    }

    /// Updates [`RectKit::screenspace`] in **screenspace** coordinates to match 
    /// the current [`RectKit::worldspace`] in **worldspace** coordinates.  
    /// \
    /// Requires the `Window`'s dimensions.
    pub fn world_to_screen(&mut self, window_width: f32, window_height: f32) -> &mut Self {
        self.screenspace = Rect::from_center_size(
            Vec2::new(
            (window_width / 2.0) + self.worldspace.center().x,
            (window_height / 2.0) - self.worldspace.center().y
            ),
            self.worldspace.size()
        );
        self
    }

    /// Updates [`RectKit::relative_worldspace`] in relative coordinates to match 
    /// the current [`RectKit::worldspace`] in **worldspace** coordinates. 
    /// Relative **worldspace** coordinates go from `-0.5` to `0.5` relative to the total size of the `Window`.  
    /// \
    /// Requires the `Window`'s dimensions.
    pub fn world_to_relative(&mut self, window_width: f32, window_height: f32) -> &mut Self {
        self.relative_worldspace = Rect::new(
            self.worldspace.min.x / window_width, 
            self.worldspace.min.y / window_height, 
            self.worldspace.max.x / window_width, 
            self.worldspace.max.y / window_height
        );
        self
    }

    /// Updates [`RectKit::worldspace`] in **worldspace** coordinates to match 
    /// the current [`RectKit::screenspace`] in **screenspace** coordinates.  
    /// \
    /// Requires the `Window`'s dimensions.
    pub fn screen_to_world(&mut self, window_width: f32, window_height: f32) -> &mut Self {
        self.worldspace = Rect::from_center_size(
            Vec2::new(
            self.screenspace.center().x - (window_width / 2.0),
            (window_height / 2.0) - self.screenspace.center().y
            ),
            self.screenspace.size()
        );
        self
    }

    /// Updates [`RectKit::relative_screenspace`] in relative coordinates to match 
    /// the current [`RectKit::screenspace`] in **screenspace** coordinates. 
    /// Relative **screenspace** coordinates go from `0.0` to `1.0` relative to the total size of the `Window`.  
    /// \
    /// Requires the `Window`'s dimensions.
    pub fn screen_to_relative(&mut self, window_width: f32, window_height: f32) -> &mut Self {
        self.relative_screenspace = Rect::new(
            self.screenspace.min.x / window_width, 
            self.screenspace.min.y / window_height, 
            self.screenspace.max.x / window_width, 
            self.screenspace.max.y / window_height
        );
        self
    }

    /// Updates the [`RectKit::worldspace`] in **worldspace** coordinates to match 
    /// the current [`RectKit::relative_worldspace`] in relative coordinates. 
    /// Relative **worldspace** coordinates go from `-0.5` to `0.5` relative to the total size of the `Window`.  
    /// \
    /// Requires the `Window`'s dimensions.
    pub fn relative_to_world(&mut self, window_width: f32, window_height: f32) -> &mut Self {
        self.worldspace = Rect::new(
            self.relative_worldspace.min.x * window_width, 
            self.relative_worldspace.min.y * window_height,
            self.relative_worldspace.max.x * window_width, 
            self.relative_worldspace.max.y * window_height
        );
        self
    }

    /// Updates [`RectKit::screenspace`] in **screenspace** coordinates to match 
    /// the current [`RectKit::relative_screenspace`] in relative coordinates. 
    /// Relative **screenspace** coordinates go from `0.0` to `1.0` relative to the total size of the `Window`.  
    /// \
    /// Requires the `Window`'s dimensions.
    pub fn relative_to_screen(&mut self, window_width: f32, window_height: f32) -> &mut Self {
        self.screenspace = Rect::new(
            self.relative_screenspace.min.x / window_width, 
            self.relative_screenspace.min.y / window_height, 
            self.relative_screenspace.max.x / window_width, 
            self.relative_screenspace.max.y / window_height
        );
        self
    }

    /// Checks to see if [`RectKit::worldspace`] is inside a window's **worldspace** [`Rect`].  
    /// \
    /// Be sure to pass in the dimensions of the correct `Window`!
    pub fn is_inside_worldspace_window(&self, window_width: f32, window_height: f32) -> bool {

        let window_rect = Rect::from_center_size(
            Vec2::ZERO, 
            Vec2::new(window_width,window_height)
        );

        window_rect.contains(self.worldspace().min) && window_rect.contains(self.worldspace().max)
    }

    /// Checks to see if the [`RectKit::screenspace`] is inside a window's **screenspace** [`Rect`].  
    /// \
    /// Be sure to pass in the dimensions of the correct `Window`!
    pub fn is_inside_screenspace_window(&self, window_width: f32, window_height: f32) -> bool {

        let window_rect = Rect::from_corners(
            Vec2::ZERO, 
            Vec2::new(window_width,window_height)
        );

        window_rect.contains(self.screenspace().min) && window_rect.contains(self.screenspace().max)
    }
}

/// Common functionality between the directional tab bars.
/// We keep the tab bars as separate components for query granularity.
pub trait TabTrim {
    
}

/// Northern border area of the [`Territory`] that hosts the feature tabs.
#[derive(Component, Clone, Copy)]
pub struct NorthTabs {

}

impl TabTrim for NorthTabs {

}

/// Eastern border area of the [`Territory`] that hosts the feature tabs.
#[derive(Component, Clone, Copy)]
pub struct EastTabs {

}

impl TabTrim for EastTabs {

}

/// Southern border area of the [`Territory`] that hosts the feature tabs.
#[derive(Component, Clone, Copy)]
pub struct SouthTabs {

}

impl TabTrim for SouthTabs {

}

/// Western border area of the [`Territory`] that hosts the feature tabs.
#[derive(Component, Clone, Copy)]
pub struct WestTabs {

}

impl TabTrim for WestTabs {

}

/// Contains every [`Territory`] [`Entity`] neighbor that this one is linked to, separated by what side they're linked on.  
///   
/// Used for graph traversals when handling linked [`DragRequest`]s.
#[derive(Component)]
pub struct CardinalConnections {
    pub northern: Vec<Entity>,
    pub eastern: Vec<Entity>,
    pub southern: Vec<Entity>,
    pub western: Vec<Entity>,
}

impl Default for CardinalConnections {
    fn default() -> Self {
        CardinalConnections { northern: Vec::new(), eastern: Vec::new(), southern: Vec::new(), western: Vec::new() }
    }
}

impl CardinalConnections {
    /// Gets the northern connections in an [`Entity`] [`Vec`].
    pub fn northern(&self) -> Vec<Entity> {
        self.northern.clone()
    }

    /// Gets the eastern connections in an [`Entity`] [`Vec`].
    pub fn eastern(&self) -> Vec<Entity> {
        self.eastern.clone()

    }
    /// Gets the southern connections in an [`Entity`] [`Vec`].
    pub fn southern(&self) -> Vec<Entity> {
        self.southern.clone()
    }

    /// Gets the western connections in an [`Entity`] [`Vec`].
    pub fn western(&self) -> Vec<Entity> {
        self.western.clone()
    }

    /// Get a [`Vec`] of every [`Entity`] linked.
    pub fn get_all_vec(&self) -> Vec<Entity> {
        let mut total_vec = Vec::new();
        total_vec.extend(&self.northern);
        total_vec.extend(&self.eastern);
        total_vec.extend(&self.southern);
        total_vec.extend(&self.western);
        total_vec
    }

    /// Get a [`Vec`] of connections the same side as the [`ResizeDirection`] passed in. 
    ///   
    /// Corner directions get a combined [`Vec`] of the adjacent sides.
    pub fn get_resize_direction_vec(&self, resize_direction: ResizeDirection) -> Vec<Entity> {
        let mut result_vec = Vec::new();
        match resize_direction {
            ResizeDirection::North => { result_vec.extend(&self.northern) },
            ResizeDirection::NorthEast => { result_vec.extend(&self.northern); result_vec.extend( &self.eastern) },
            ResizeDirection::East => { result_vec.extend(&self.eastern) },
            ResizeDirection::SouthEast => { result_vec.extend(&self.southern); result_vec.extend(&self.eastern) },
            ResizeDirection::South => { result_vec.extend(&self.southern) },
            ResizeDirection::SouthWest => { result_vec.extend(&self.southern); result_vec.extend(&self.western) },
            ResizeDirection::West => { result_vec.extend(&self.western) },
            ResizeDirection::NorthWest => { result_vec.extend(&self.northern); result_vec.extend(&self.western) }
        }
        result_vec
    }
}

/// Marker component for group of [`Territory`]s separated for drag querying.
#[derive(Component)]
pub struct DragTerritoryGroup;

/// Marker component for group of [`Territory`]s separated for resize querying.
#[derive(Component)]
pub struct AdvancingTerritoryGroup;

/// Marker component for group of [`Territory`]s separated for resize querying.
#[derive(Component)]
pub struct RetreatingTerritoryGroup;

/// Identifies entity as a [`Territory`] UI element. A [`Territory`] can be moved and resized, 
/// but cannot overlap with other [`Territory`]s.  
/// \
/// [`Territory`]s define a space in which [`Tab`]s are organized and display their content.
#[derive(Component)]
pub struct Territory {
    /// Collection of [`Rect`]s describing the [`Territory`]'s location in the `Window`.
    pub expanse: RectKit,
    /// [`Entity`] ID of the base container node, covering the entire size of the [`Territory`].
    pub base_node: Option<Entity>,
    /// [`Entity`] ID of the node area where the [`Territory`] will sense drag interactions.
    pub drag_node: Option<Entity>,
    /// [`Entity`] ID of the base resize grid node.
    pub resize_node: Option<Entity>

}
impl Default for Territory {
    fn default() -> Self {
        Territory {
            expanse: RectKit::default(),
            base_node: None,
            drag_node: None,
            resize_node: None
        }
    }
}
impl Territory {
    pub fn new(
        expanse: RectKit,
        base_node: Option<Entity>,
        drag_node: Option<Entity>,
        resize_node: Option<Entity>
    ) -> Self {
            Territory { expanse, base_node, drag_node, resize_node }
        }

    /// Creates a [`Territory`] with all zero-sized [`Rect`]s.
    pub fn empty() -> Self {
        Territory { expanse: RectKit::empty(), ..default() }
    }

    /// Gets the [`RectKit`] containing all of the location [`Rect`]s. 
    pub fn expanse(&self) -> RectKit {
        self.expanse
    }

    /// Gets the current base node.
    pub fn base_node(&self) -> Option<Entity> {
        self.base_node
    }

    /// Gets the current drag node.
    pub fn drag_node(&self) -> Option<Entity> {
        self.drag_node
    }

    /// Gets the current resize nodes.
    pub fn resize_node(&self) -> Option<Entity> {
        self.resize_node
    }

}

/// The intended movement behavior [`MoveRequest`] wants.
#[derive(Clone, Copy)]
pub enum MoveRequestType {
    /// Some display libraries are unable to send information about if the UI element is being dragged or resized.
    /// [`MoveRequest`] processing systems will determine the movement type if handed a [`MoveRequestType::Unknown`].
    Unknown,
    /// The movement changes the [`Rect`]'s position but not its size.
    Drag,
    /// The movement changes the [`Rect`]'s size and/or position. Wraps around a ResizeDirection enum.
    Resize(ResizeDirection)
}

/// Marks a [`TerritoryTabs`] UI element as having been commanded to move without changing size. Entities with this component will be processed 
/// by motion systems and this component will be removed once all processing is complete.
#[derive(Component, Clone)]
pub struct DragRequest {
    /// Collection of [`Rect`]s describing the [`DragRequest`]'s proposed location in the `Window`.
    pub proposed_expanse: RectKit,
    /// Drag vector in **screenspace** coordinates. Flip the y to get a worldspace delta.
    pub drag_delta: Vec2
}

impl Default for DragRequest {
    fn default() -> Self {
        DragRequest { proposed_expanse: RectKit::default(), drag_delta: Vec2::ZERO }
    }
}

impl DragRequest {
    pub fn new (proposed_expanse: RectKit, drag_delta: Vec2) -> Self {
        DragRequest { proposed_expanse, drag_delta }
    }

    /// Gets the [`RectKit`] containing the proposed [`Rect`]s UI element wants to move to.
    pub fn proposed_expanse(&self) -> RectKit { self.proposed_expanse }

    /// Gets the delta of the drag movement in **screenspace** coordinates.
    pub fn drag_delta(&self) -> Vec2 { self.drag_delta }
}

/// Marks a [`TerritoryTabs`] UI element as having been commanded to change its boundaries. Entities with this component will be processed 
/// by motion systems and this component will be removed once all processing is complete.
#[derive(Component, Clone)]
pub struct ResizeRequest {
    /// Collection of [`Rect`]s describing the [`DragRequest`]'s proposed location in the `Window`.
    pub proposed_expanse: RectKit,
    /// What resize handle did the user manipulate to command this motion?
    pub resize_direction: ResizeDirection
}

impl Default for ResizeRequest {
    fn default() -> Self {
        ResizeRequest { 
            proposed_expanse: RectKit::default(), 
            resize_direction: ResizeDirection::West
        }
    }
}

impl ResizeRequest {
    pub fn new (proposed_expanse: RectKit, resize_direction: ResizeDirection) -> Self {
        ResizeRequest { 
            proposed_expanse,
            resize_direction
        }
    }

    /// Gets the [`RectKit`] containing the proposed [`Rect`]s UI element wants to move to.
    pub fn proposed_expanse(&self) -> RectKit { self.proposed_expanse }
}






/// Marks a [`TerritoryTabs`] UI element as having been commanded to move. Entities with this component will be processed 
/// by motion systems and this component will be removed once all processing is complete.
#[derive(Component, Clone)]
pub struct MoveRequest {
    /// Collection of [`Rect`]s describing the [`MoveRequest`]'s proposed location in the `Window`.
    pub proposed_expanse: RectKit,
    /// Kind of movement that will inform processing systems.
    pub move_type: MoveRequestType
}

impl Default for MoveRequest {
    fn default() -> Self {
        MoveRequest {
            proposed_expanse: RectKit::default(),
            move_type: MoveRequestType::Unknown
        }
    }
}

impl MoveRequest {
    pub fn new (
        proposed_expanse: RectKit,
        move_type: MoveRequestType
    ) -> Self {
        MoveRequest {
            proposed_expanse,
            move_type
        }
    }

    /// Gets the [`RectKit`] containing the proposed [`Rect`]s UI element wants to move to.
    pub fn proposed_expanse(&self) -> RectKit {self.proposed_expanse}

    /// Gets the [`MoveRequestType`] this [`MoveRequest`] component is set to.
    pub fn move_type(&self) -> MoveRequestType {self.move_type}

    /// Changes the `move_type` to `Unknown`, meaning we don't yet have the information to know what this component wants.
    pub fn move_type_unknown(&mut self) -> &mut Self {
        self.move_type = MoveRequestType::Unknown;
        self
    }

    /// Changes the `move_type` to `Drag`, marking the UI element as moving without changing size.
    pub fn move_type_drag(&mut self) -> &mut Self {
        self.move_type = MoveRequestType::Drag;
        self
    }

    /// Changes the `move_type` to `Resize`, marking the UI element as changing size in a specific direction.
    pub fn move_type_resize(&mut self, resize_direction: ResizeDirection) -> &mut Self {
        self.move_type = MoveRequestType::Resize(resize_direction);
        self
    }
}

/// Replacement for placeholders and overlays.
pub struct Glance {

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn territory_translates_correctly_from_screenspace() {
        let mut test_terr = Territory::empty();
        let input_screen = Rect::new(0.0, 0.0, 100.0, 100.0);
        let output_world = Rect::new(-500.0, 400.0, -400.0, 500.0);
        let output_rel_world = Rect::new(-0.5, 0.4, -0.4, 0.5);
        let output_rel_screen = Rect::new(0.0, 0.0, 0.1, 0.1);
        test_terr.expanse.set_screenspace(
            input_screen,
            1000.0, 
            1000.0
        );
        assert_eq!(
            test_terr.expanse.worldspace(), 
            output_world, 
            "Set screen failed translate to worldspace rect."
        );
        assert_eq!(
            test_terr.expanse.relative_worldspace(), 
            output_rel_world, 
            "Set screen failed translate to relative worldspace rect."
        );
        assert_eq!(
            test_terr.expanse.relative_screenspace(), 
            output_rel_screen, 
            "Set screen failed translate to relative screenspace rect."
        );
    }

    #[test]
    fn territory_translates_correctly_from_worldspace() {
        let mut test_terr = Territory::empty();
        let input_world = Rect::new(-50.0, -50.0, 50.0, 50.0);
        let output_screen = Rect::new(450.0, 450.0, 550.0, 550.0);
        let output_rel_screen = Rect::new(0.45, 0.45, 0.55, 0.55);
        let output_rel_world = Rect::new(-0.05, -0.05, 0.05, 0.05);
        test_terr.expanse.set_worldspace(
            input_world,
            1000.0, 
            1000.0
        );
        assert_eq!(
            test_terr.expanse.screenspace(), 
            output_screen, 
            "Set world failed translate to screenspace rect."
        );
        assert_eq!(
            test_terr.expanse.relative_screenspace(), 
            output_rel_screen, 
            "Set world failed translate to relative screenspace rect."
        );
        assert_eq!(
            test_terr.expanse.relative_worldspace(), 
            output_rel_world, 
            "Set world failed translate to relative worldspace rect."
        );
    }

    #[test]
    fn territory_movement_methods_move_correctly() {
        let mut test_terr = Territory::empty();
        test_terr.expanse.set_screenspace(
            Rect::new(0.0, 0.0, 100.0, 100.0), 
            1000.0, 
            1000.0
        );
        test_terr.expanse.move_screenspace_pos(500.0, 500.0, 1000.0, 1000.0);
        assert_eq!(
            test_terr.expanse.screenspace(),
            Rect::new(500.0, 500.0, 600.0, 600.0),
            "Move screen pos failure."
        );
        test_terr.expanse.move_screenspace_corners(
            Vec2::new(-100.0, -100.0), 
            Vec2::new(-100.0, -100.0), 
            1000.0, 
            1000.0
        );
        assert_eq!(
            test_terr.expanse.screenspace(),
            Rect::new(400.0, 400.0, 500.0, 500.0),
            "Move screen corners failure."
        );

        test_terr.expanse.set_worldspace(
            Rect::new(-100.0, -100.0, 100.0, 100.0), 
            1000.0, 
            1000.0
        );
        test_terr.expanse.move_worldspace_pos(
            100.0, 
            100.0, 
            1000.0, 
            1000.0
        );
        assert_eq!(
            test_terr.expanse.worldspace(),
            Rect::new(0.0, 0.0, 200.0, 200.0),
            "Move world pos failure."
        );
        test_terr.expanse.move_worldspace_corners(
            Vec2::new(-100.0, -100.0), 
            Vec2::new(-100.0, -100.0), 
            1000.0, 
            1000.0
        );
        assert_eq!(
            test_terr.expanse.worldspace(),
            Rect::new(-100.0, -100.0, 100.0, 100.0),
            "Move world corners failure."
        );
    }
}