use bevy::prelude::*;

/// Name component for debugging.
#[derive(Component)]
pub struct Name(pub String);

/// Cleanup markers for triggering cleanup systems.
#[derive(Component)]
pub struct CleanupOnWindowClose;
    
#[derive(Component)]
pub struct CleanupOnMovingTabExit;

