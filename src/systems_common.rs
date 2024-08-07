use bevy::prelude::*;

/// Cleanup markers for triggering cleanup systems.
#[derive(Component)]
pub struct CleanupOnWindowClose;
    
#[derive(Component)]
pub struct CleanupOnMovingTabExit;


// App states.
// Territory Tabs states.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TerritoryTabsState {
    #[default]
    Empty, // No territories exist. Need to make one before the user can do anything.
    Natural, // Running normally. Operating the functions in Tabs.
    MovingTabs, // A Tab move is underway! No Territories will be moved, and normal Tab operations are disabled.
    DraggingTerritories, // User is dragging a Territory. Run collision detection and disable Tab operations.
    ResizingTerritories, // User is resizing a Territory. Run resize logic and disable Tab operations.
    LoadingLayouts // User is loading in a saved layout of Territories & Tabs. Existing ones cannot be interacted with.
}

/// Remove all entities with a specified component.
pub fn despawn_all_entities_with<T: Component> (
    mut commands: Commands,
    cleanup_query: Query<Entity, With<T>>
) {
    cleanup_query.iter().for_each(|target| {commands.entity(target).despawn_recursive();});
}

/// Remove all existing components of type T.
pub fn remove_all_components_of_type<T: Component> (
    mut commands: Commands,
    cleanup_query: Query<Entity, With<T>>
) {
    cleanup_query.iter().for_each(|target| {commands.entity(target).remove::<T>();});
}