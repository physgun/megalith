use bevy::input::mouse;
use bevy::prelude::*;
use bevy::window::*;
use bevy::render::camera::*;

use crate::components_ui::*;
use crate::input_manager::*;
use crate::resources_ui::*;
use crate::systems_common::*;
use crate::systems_egui::*;

use std::f32::consts::FRAC_PI_4;
use std::f32::consts::PI;
use std::ops::{Add, Sub};

// Load in all the ui stuff.
pub fn initialize_ui_resources (mut commands: Commands) {
    commands.init_resource::<TerritorySettings>();
    commands.init_resource::<WorldMousePosition>();
}

// Get the World Space coordinates of the mouse, 
// and optionally the window / territory / tab it is in.
// Runs all of the time. Why does everything need different coordinate systems??
pub fn get_mouse_location(
    mut mouse_location_resource: ResMut<WorldMousePosition>,
    cameras_query: Query<(&Camera, &GlobalTransform), With<MouseSeekingCamera>>,
    windows_query: Query<(Entity, &Window)>,
    territories_query: Query<(Entity, &Parent, &Territory)>,
    // TODO: Tab query here later!
    mut gizmos: Gizmos
) {
    for (camera, camera_transform) in & cameras_query {    
        for (entity_window, window) in & windows_query {
            match camera.target {
                RenderTarget::Window(WindowRef::Entity(entity)) => {
                    if let Some(camera_mouse_position) = window.cursor_position()
                        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
                        .map(|ray| ray.origin.truncate()) {

                        mouse_location_resource.pos = camera_mouse_position;
                        mouse_location_resource.window = Some(entity);

                        gizmos.line_2d(Vec2::ZERO, mouse_location_resource.pos, Color::WHITE);
                        
                        for (entity_territory, parent, territory) in & territories_query {
                            if parent.get() == entity && territory.rect.contains(mouse_location_resource.pos) {
                                mouse_location_resource.territory = Some(entity_territory);
                                gizmos.rect_2d(
                                    territory.rect.center(), 
                                    0.0, 
                                    territory.rect.size().add(Vec2::new(10.0, 10.0)), 
                                    Color:: GOLD
                                );
                            }
                        }
                    }
                    else {
                        mouse_location_resource.window = None;
                        mouse_location_resource.territory = None;
                        mouse_location_resource.tab = None;
                    }
                    }
                _ => {}
            }
        }
    }
}

// A default configuration for the OS windows. Background camera, names, etc.
// Summoned by a WindowCreated event and configures that exact window.
pub fn configure_os_window(
    mut commands: Commands,
    mut window_spawn_detected_events: EventReader<WindowCreated>,
    mut windows_query: Query<(Entity, &mut Window)>
) {
    for event in window_spawn_detected_events.read() {
        for (entity, mut window) in &mut windows_query {
            if entity == event.window{
                window.title = "Territory Tabs".to_string();

                let child_camera = commands
                .spawn((
                    EntityName("[CAMERA] UI Camera".to_string()),
                    CleanupOnWindowClose,
                    Camera2dBundle {
                        camera: Camera {
                            clear_color: ClearColorConfig::Custom(Color::rgb(0.29, 0.29, 0.29)), 
                            target: RenderTarget::Window(WindowRef::Entity(entity)),
                            ..Default::default() 
                            }, 
                        ..Default::default()
                        },
                        MouseSeekingCamera
                    ))
                .id();
        
                // Add camera as child to the window and give additional components.
                commands.entity(entity)
                .add_child(child_camera)
                .insert((
                    EntityName("[WINDOW] Territory Tabs Window".to_string()),
                    TerritoryTabsUI,
                    EguiDisplay,
                    SpatialBundle{..Default::default()}
                ));
            }
        }
    }
}

// Spawns a new window on a dev command for testing.
pub fn spawn_new_os_window(
    mut commands: Commands,
    mut spawn_window_button_events: EventReader<SpawnWindowKeyJustPressed>
) {
    for event in spawn_window_button_events.read() {
        commands.spawn(Window::default());
    }
}

// Listen for Tab getting picked up and spawn in the placeholders.
// We start with a TabMove-type and a TabOrigin-type placeholder.
// The TabOrigin will remain in the starting Territory the whole time. 
// The TabMove will be changed and kicked around as mouse behavior dictates.
pub fn setup_tab_move_placeholders(
    mut commands: Commands,
    mouse_location_resource: Res<WorldMousePosition>,
    territory_query: Query<&Territory>
) {
    if let Some(window_entity) = mouse_location_resource.window {

        // This is a special situation during debugging when no territories exist.
        // The special spawn button for when this happens hasn't been implemented yet.   
        if territory_query.is_empty() {
            let starter_territory = commands.spawn((
                EntityName("[PLACEHOLDER] Starter Territory".to_string()),
                CleanupOnMovingTabExit,
                Placeholder {placeholder_type: PlaceholderType::SpawnTerritory, ..Default::default()},
                SpatialBundle::default(),
            ))  .id();
            commands.entity(window_entity).add_child(starter_territory);
            debug!("Spawned special starter placeholder of type: SpawnTerritory");

            return;
        }

        let tab_move = commands.spawn((
            EntityName("[PLACEHOLDER] Initial TabMove".to_string()),
            CleanupOnMovingTabExit,
            Placeholder {placeholder_type: PlaceholderType::TabMove, ..Default::default()},
            SpatialBundle::default(),
        ))  .id();
        commands.entity(window_entity).add_child(tab_move);
        debug!("Spawned placeholder of type: TabMove");
        let tab_origin = commands.spawn((
            EntityName("[PLACEHOLDER] Initial TabOrigin".to_string()),
            CleanupOnMovingTabExit,
            Placeholder {placeholder_type: PlaceholderType::TabOrigin, ..Default::default()},
            SpatialBundle::default(),
        ))  .id();
        commands.entity(window_entity).add_child(tab_origin);
        debug!("Spawned placeholder of type: TabOrigin");

        // TODO: These need to be children of the Territory we started from instead of the Window.
    }
    else {warn!("Mouse window not found at start of Tab Move! No placeholders spawned!");}
}

// TerritoryTab state machine. Is sent events to change state?
// For now the dev chord events trigger this. 
pub fn territory_tabs_state_change (
    territory_tabs_current_state: Res<State<TerritoryTabsState>>,
    mut territory_tabs_next_state: ResMut<NextState<TerritoryTabsState>>,
    mut entered_moving_tabs_events: EventReader<TestChordJustPressed>,
    mut exited_moving_tabs_events: EventReader<TestChordJustReleased>
) {
    for event in entered_moving_tabs_events.read() {
        match territory_tabs_current_state.get() {
            TerritoryTabsState::Natural => territory_tabs_next_state.set(TerritoryTabsState::MovingTabs),
            _ => {warn!("Failed to switch Territory Tabs state to MovingTabs!");}
        }
    }
    for event in exited_moving_tabs_events.read() {
        match territory_tabs_current_state.get() {
            TerritoryTabsState::MovingTabs => territory_tabs_next_state.set(TerritoryTabsState::Natural),
            _ => {warn!("Failed to switch Territory Tabs state to Natural!");}
        }
    }
}


// See if the mouse has triggered any events while the tab move event is ongoing.
// Subject to in_state run condition, only runs when a tab move is underway.
// Move placeholders and update placeholder types if necessary.

// Check for the cursor leaving the window.
// Despawn any TabMove and SpawnTerritory placeholders.
pub fn check_placeholder_types_leaving_window (
    mut commands: Commands,
    mut mouse_left_window_events: EventReader<CursorLeft>,
    mut placeholder_query: Query<(Entity, &mut Placeholder)>
) {
    for event in mouse_left_window_events.read() {
        for (entity, placeholder) in &mut placeholder_query {
            match placeholder.placeholder_type {
                PlaceholderType::SpawnTerritory => {
                    commands.entity(event.window).remove_children(&[entity]);
                    commands.entity(entity).despawn();
                    debug!("[CURSOR LEFT] Removed SpawnTerritory type placeholder!"); 
                }
                PlaceholderType::TabMove => {
                    commands.entity(event.window).remove_children(&[entity]);
                    commands.entity(entity).despawn();
                    debug!("[CURSOR LEFT] Removed TabMove type placeholder!");

                } // TODO: Update to Tab's Territory instead of Window
                PlaceholderType::SpawnWindow => {
                    warn!("[CURSOR LEFT] SpawnWindow type placeholder found while mouse was still in a Window??"); 
                    commands.entity(entity).despawn();
                }
                _ => {} // Leave others alone.
            };
        }
        // Add a SpawnWindow placeholder.
        commands.spawn((
            EntityName("[PLACEHOLDER] CursorLeft Event SpawnWindow".to_string()),
            CleanupOnMovingTabExit,
            Placeholder {placeholder_type: PlaceholderType::SpawnWindow, ..Default::default()},
            SpatialBundle::default(),
        ));
        debug!("[CURSOR LEFT] Spawned a SpawnWindow type placeholder!");
    }
}

// Check for the cursor entering a new window.
// Remove any SpawnWindow type placeholders.
pub fn check_placeholder_types_entering_window (
    mut commands: Commands,
    mut mouse_entered_window_events: EventReader<CursorEntered>,
    mut placeholder_query: Query<(Entity, &mut Placeholder)>
) {
    for event in mouse_entered_window_events.read() {
        for (entity, placeholder) in &mut placeholder_query {
            match placeholder.placeholder_type {
                PlaceholderType::SpawnTerritory => {
                    warn!("[CURSOR ENTERED] Removed SpawnTerritory type placeholder, found while not in a Window??");
                    commands.entity(entity).despawn();
                }
                PlaceholderType::TabMove => {
                    warn!("[CURSOR ENTERED] Removed TabMove type placeholder, found while not in a Window??");
                    commands.entity(entity).despawn();
                }
                PlaceholderType::SpawnWindow => {
                    commands.entity(entity).despawn();
                    debug!("[CURSOR ENTERED] Removed SpawnWindow type placeholder!");
                }
                _ => {} // Leave others alone.
            };
        }
        // Spawn a new child placeholder. SpawnTerritory type since calculate_placeholder_data will catch it.
        let new_placeholder = commands.spawn((
            EntityName("[PLACEHOLDER] CursorEntered Event SpawnTerritory".to_string()),
            CleanupOnMovingTabExit,
            Placeholder {placeholder_type: PlaceholderType::SpawnTerritory, ..Default::default()},
            SpatialBundle::default()
        ))  .id();
        commands.entity(event.window).add_child(new_placeholder);
        debug!("[CURSOR ENTERED] Spawned new placeholder of type SpawnTerritory");
    }
}

// Check for mouse movement in the Window we're in.
// If so, see if we're in a Territory and change placeholder type.
pub fn check_placeholder_types_on_mouse_move (
    mut commands: Commands,
    mouse_location_resource: Res<WorldMousePosition>,
    mut mouse_moved_in_window_events: EventReader<CursorMoved>,
    mut placeholder_query: Query<(Entity, &mut Placeholder)>,
    territory_query: Query<(&Parent, &Territory)>
) {
    for event in mouse_moved_in_window_events.read() {
        for (parent, territory) in & territory_query {
            let territory_window = parent.get();
            if territory_window == event.window && territory.rect.contains(mouse_location_resource.pos) {
                for (entity, mut placeholder) in &mut placeholder_query {
                    match placeholder.placeholder_type {
                        PlaceholderType::SpawnTerritory => {
                            placeholder.placeholder_type = PlaceholderType::TabMove;
                            debug!("[CURSOR MOVED] Changed placeholder type from SpawnTerritory to TabMove!");
                        }// TODO: Move child into Territory's children.
                        PlaceholderType::TabMove => {}
                        PlaceholderType::SpawnWindow => {
                            warn!("[CURSOR MOVED] SpawnWindow type placeholder found inside of a Window??");
                            commands.entity(entity).despawn();
                        }
                        _ => {} // Leave others alone.
                    };
                }
            }
            else {
                for (entity, mut placeholder) in &mut placeholder_query {
                    match placeholder.placeholder_type {
                        PlaceholderType::SpawnTerritory => {}
                        PlaceholderType::TabMove => {
                            placeholder.placeholder_type = PlaceholderType::SpawnTerritory;
                            debug!("[CURSOR MOVED] Changed placeholder type from TabMove to SpawnTerritory!");
                        }
                        PlaceholderType::SpawnWindow => {
                            warn!("[CURSOR MOVED] SpawnWindow type placeholder found inside of a Window??");
                            commands.entity(entity).despawn();}
                        _ => {} // Leave others alone.
                    };
                }
            }
        }
    }
}

// With the tab movement state ongoing, check for mouse movement.
// Calculate the visual_rects of the placeholders and determine spawn validity.
// Subject to on_event run condition, only runs when a tab move is underway.
pub fn calculate_placeholder_data(
    mut gizmos: Gizmos,
    mouse_location_resource: Res<WorldMousePosition>,
    territory_settings: Res<TerritorySettings>,
    mut mouse_moved_in_window_events: EventReader<CursorMoved>,
    window_query: Query<&Window>,
    territory_query: Query<(&Parent, &Territory)>,
    mut placeholder_query: Query<&mut Placeholder>
) {
    for event in mouse_moved_in_window_events.read() {
        if let Ok(window) = window_query.get(event.window) {
            // Get the event window's size for later. World coordinates!
            let window_rect = Rect::from_center_size(
                Vec2::new(0.0, 0.0),
                Vec2::new(window.width(), window.height())
            );
            for mut placeholder in &mut placeholder_query {
                match placeholder.placeholder_type {
                    PlaceholderType::SpawnTerritory => {
                        // Get upper left coord. Adjust slightly for tab_offsets.
                        // World coordinates!
                        let upper_left = Vec2::new(
                            mouse_location_resource.pos.x - territory_settings.tab_offset.x,
                            mouse_location_resource.pos.y + territory_settings.tab_offset.y
                        );

                        // Get the initial minimum and default territory rects.
                        let mut proposed_rects = vec![
                            Rect::from_corners(
                                upper_left, 
                                Vec2::new(
                                    upper_left.x + territory_settings.min_size.x,
                                    upper_left.y - territory_settings.min_size.y
                                )
                            ),
                            Rect::from_corners(
                                upper_left, 
                                Vec2::new(
                                    upper_left.x + territory_settings.default_size.x,
                                    upper_left.y - territory_settings.default_size.y
                                )
                            )];

                        // Clip off anything outside the window.
                        proposed_rects[1] = window_rect.intersect(proposed_rects[1]);

                        // Intersecting territories clip off pieces of our initial default rect too.
                        for (parent, territory) in &territory_query {
                            let territory_conflict = proposed_rects[1].intersect(territory.rect);
                            let territory_window = parent.get();
                            if territory_window == event.window && !territory_conflict.is_empty() {
                                gizmos.rect_2d(
                                    territory_conflict.center(), 
                                    0.0,
                                    territory_conflict.size(),
                                    Color::RED
                                );
                            
                                let conflict_angle = (upper_left.y - territory.rect.center().y)
                                    .atan2(upper_left.x - territory.rect.center().x);

                                if conflict_angle <= FRAC_PI_4 && conflict_angle >= -FRAC_PI_4 {
                                    proposed_rects[1].min.x += territory_conflict.width();
                                    gizmos.line_2d(territory.rect.center(), upper_left, Color::GREEN);
                                } 
                                else if conflict_angle >= FRAC_PI_4 && conflict_angle <= 3.0 * FRAC_PI_4 {
                                    proposed_rects[1].min.y += territory_conflict.height();
                                    gizmos.line_2d(territory.rect.center(), upper_left, Color::BLUE);
                                }
                                else if (conflict_angle >= 3.0 * FRAC_PI_4 && conflict_angle <= PI)
                                    || (conflict_angle >= -PI && conflict_angle <= -3.0 * FRAC_PI_4) {
                                    proposed_rects[1].max.x -= territory_conflict.width();
                                    gizmos.line_2d(territory.rect.center(), upper_left, Color::YELLOW);
                                }
                                else if conflict_angle >= -3.0 * FRAC_PI_4 && conflict_angle <= -FRAC_PI_4 {
                                    proposed_rects[1].max.y -= territory_conflict.height();
                                    gizmos.line_2d(territory.rect.center(), upper_left, Color::RED);
                                }
                                else{
                                    warn!{"Unusual conflict angle found during placeholder calculations!"}
                                }
                            }
                        }
                        // If the minimum still fits inside the clipped default, we're good to spawn.
                        // If not, ignore this frame's data to keep the last valid data.
                        if proposed_rects[1].contains(proposed_rects[0].min) 
                        && proposed_rects[1].contains(proposed_rects[0].max) {
                            placeholder.visual_rects = proposed_rects;
                            placeholder.valid_spawn = true;
                        }
                        
                    }
                    PlaceholderType::TabMove => {} // Do this later.
                    _ =>{}
                }
            }
        }
        else {warn!("Unable to get the window from the CursorMoved event!")}
    }
}


/// Iterate through all placeholders, and do what actions they represent.
pub fn activate_placeholders (
    mut commands: Commands,
    mouse_location_resource: Res<WorldMousePosition>,
    placeholders_query: Query<(Entity, Option<&Parent>, &Placeholder)>
) {
    for (entity, parent, placeholder) in & placeholders_query {
        match placeholder.placeholder_type {
            PlaceholderType::SpawnTerritory => {
                if let Some(territory_parent) = parent {
                    if placeholder.valid_spawn {
                        if let Some(mouse_window) = mouse_location_resource.window { 
                            let new_territory = commands.spawn((
                                EntityName("[TERRITORY] Spawned By Placeholder".to_string()),
                                CleanupOnWindowClose,
                                Territory {
                                    rect: placeholder.visual_rects[1],
                                    ..Default::default()
                                },
                                SpatialBundle::default(),
                            ))  .id();
                            commands.entity(mouse_window).add_child(new_territory);
                        }
                        else {warn!("Attempted to activate SpawnTerritory, but no mouse window found!");}
                    }
                }
                else{warn!("SpawnTerritory type placeholder found without window parent!");}
            },
            PlaceholderType::TabMove => {
                debug!("TabMove type placeholder activated! Pretend that a tab move occured.");
            },
            PlaceholderType::TabOrigin => {
                debug!("TabOrigin type placeholder activated! Pretend that nothing happened.");
            },
            PlaceholderType::CombineTerritories => {
                warn!("Unimplemented CombineTerritories type placeholder activated!");
            },
            PlaceholderType::SpawnWindow => {
                debug!("SpawnWindow type placeholder activated! Pretend that a window spawned.");
            },
            PlaceholderType::LoadLayout => {
                warn!("Unimplemented LoadLayout type placeholder activated!");
            }
        }
    }
}

// With tab movement state just exited, use the placeholders to figure out what needs to be done.
// Then, despawn them.
// TODO: This needs a serious rework. What happens when no valid spawns?
pub fn initiate_placeholders(
    mut commands: Commands,
    mut tab_move_ended_events: EventReader<TestChordJustReleased>,
    territory_query: Query<(&Parent, &Territory)>,
    mut placeholder_query: Query<(Entity, &mut Placeholder)>,
) {
    for event in tab_move_ended_events.read() {
        for (entity, mut placeholder) in &mut placeholder_query{
            match placeholder.placeholder_type {
                PlaceholderType::SpawnTerritory => {
                    if placeholder.valid_spawn {
                        let new_territory = commands.spawn((
                            Territory {
                                rect: placeholder.visual_rects[1],
                                ..Default::default()
                                },
                            SpatialBundle::default(),
                            EguiDisplay // Display Territory with egui.
                        ))  .id();
                        commands.entity(event.0).add_child(new_territory);
                        info!("Territory spawned!");
                    }
                    commands.entity(entity).despawn();
                    debug!("Placeholder of type SpawnTerritory despawned!");
                }
                PlaceholderType::TabMove => {
                    commands.entity(entity).despawn();
                    debug!("Placeholder of type TabMove despawned!");
                }
                PlaceholderType::TabOrigin => {
                    commands.entity(entity).despawn();
                    debug!("Placeholder of type TabOrigin despawned!");
                }
                PlaceholderType::LoadLayout => {
                    commands.entity(entity).despawn();
                    debug!("Placeholder of type LoadLayout despawned!");
                }
                PlaceholderType::SpawnWindow => {
                    commands.entity(entity).despawn();
                    debug!("Placeholder of type SpawnWindow despawned!");
                }
                _ => {}
            }
        }
    }
}