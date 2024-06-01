use bevy::prelude::*;

/// Cleanup markers for triggering cleanup systems.
#[derive(Component)]
pub struct CleanupOnWindowClose;
    
#[derive(Component)]
pub struct CleanupOnMovingTabExit;

