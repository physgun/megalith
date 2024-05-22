use bevy::input::mouse;
use bevy::prelude::*;
use bevy::window::*;
use bevy::render::camera::*;

use crate::components_ui::*;
use crate::events_ui::*;
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
    commands.init_resource::<TabSettings>();
    commands.init_resource::<WorldMousePosition>();
}

// Debug system displaying all the gizmos
pub fn display_debug_gizmos (
    mut gizmos: Gizmos,
    territory_query: Query<&Territory>
) {
    for territory in & territory_query {
        gizmos.rect_2d(
            territory.worldspace_rect().center(), 
            0.0,
            territory.worldspace_rect().size(),
            Color::BLUE
        );
    }
}

// Get the Screenspace / Worldspace coordinates of the mouse, 
// and optionally the window / territory / tab it is in.
// Runs all of the time. Why does everything need different coordinate systems??
pub fn get_mouse_location(
    mut mouse_location_resource: ResMut<WorldMousePosition>,
    cameras_query: Query<(&Camera, &GlobalTransform), With<MouseSeekingCamera>>,
    windows_query: Query<(Entity, &Window)>,
    territories_query: Query<(Entity, &Parent, &Territory)>,
    // TODO: Tab query here later!
) {
    // Reset mouse info so we don't keep around old data.
    // TODO: Move mouse info from resource to events
    mouse_location_resource.window = None;
    mouse_location_resource.territory = None;
    mouse_location_resource.tab = None;

    for (camera, camera_transform) in & cameras_query {    
        for (entity_window, window) in & windows_query {
            match camera.target {
                RenderTarget::Window(WindowRef::Entity(entity)) => {
                    if let Some(camera_mouse_position) = window.cursor_position()
                        .and_then(|cursor| camera.viewport_to_world_2d(
                            camera_transform, 
                            cursor))
                        .map(|ray| ray) {

                        mouse_location_resource.screenspace_pos = window.cursor_position().unwrap();
                        mouse_location_resource.worldspace_pos = camera_mouse_position;
                        mouse_location_resource.window = Some(entity);
                        
                        for (entity_territory, parent, territory) in & territories_query {
                            if parent.get() == entity 
                                && territory.worldspace_rect().contains(mouse_location_resource.worldspace_pos) {
                                mouse_location_resource.territory = Some(entity_territory);
                            }
                        }
                    }
                }
                _ => {warn!("No RenderTarget found for camera when getting mouse info!");}
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
                    TerritoryTabs,
                    DisplayLibrary::BevyEgui,
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
        commands.spawn((
            EntityName("[WINDOW] Test Spawn Window".to_string()),
            Window::default(),
            TerritoryTabs,
            DisplayLibrary::BevyEgui
        ));
    }
}

// TerritoryTabs operating state machine handling exit events first.
pub fn territory_tabs_main_state_exit (
    territory_tabs_current_state: Res<State<TerritoryTabsState>>,
    mut territory_tabs_next_state: ResMut<NextState<TerritoryTabsState>>,
    mut territory_move_tab_exit_events: EventReader<TestChordJustReleased>,
    mut territory_drag_ended_events: EventReader<TerritoryDragEnded>,
    mut territory_resize_ended_events: EventReader<TerritoryResizeEnded>
) {
    for event in territory_move_tab_exit_events.read() {
        match territory_tabs_current_state.get() {
            TerritoryTabsState::MovingTabs => territory_tabs_next_state.set(TerritoryTabsState::Natural),
            _ => {warn!("[MAIN STATE] Invalid transition: {:?} -> Natural", territory_tabs_current_state.get());}
        }
    }
    for event in territory_drag_ended_events.read() {
        match territory_tabs_current_state.get() {
            TerritoryTabsState::DraggingTerritories => territory_tabs_next_state.set(TerritoryTabsState::Natural),
            _ => {warn!("[MAIN STATE] Invalid transition: {:?} -> Natural", territory_tabs_current_state.get());}
        }
    }
    for event in territory_resize_ended_events.read() {
        match territory_tabs_current_state.get() {
            TerritoryTabsState::ResizingTerritories => territory_tabs_next_state.set(TerritoryTabsState::Natural),
            _ => {warn!("[MAIN STATE] Invalid transition: {:?} -> Natural", territory_tabs_current_state.get());}
        }
    }
}

// TerritoryTabs operation state machine handling enter events after.
pub fn territory_tabs_main_state_enter (
    territory_tabs_current_state: Res<State<TerritoryTabsState>>,
    mut territory_tabs_next_state: ResMut<NextState<TerritoryTabsState>>,
    mut territory_move_tab_enter_events: EventReader<TestChordJustPressed>,
    mut territory_drag_started_events: EventReader<TerritoryDragStarted>,
    mut territory_resize_started_events: EventReader<TerritoryResizeStarted>
) {
    for event in territory_move_tab_enter_events.read() {
        match territory_tabs_current_state.get() {
            TerritoryTabsState::Natural => territory_tabs_next_state.set(TerritoryTabsState::MovingTabs),
            _ => {warn!("[MAIN STATE] Invalid transition: {:?} -> MovingTabs", territory_tabs_current_state.get());}
        }
    }
    for event in territory_drag_started_events.read() {
        match territory_tabs_current_state.get() {
            TerritoryTabsState::Natural => territory_tabs_next_state.set(TerritoryTabsState::DraggingTerritories),
            _ => {warn!("[MAIN STATE] Invalid transition: {:?} -> DraggingTerritories", territory_tabs_current_state.get());}
        }
    }
    for event in territory_resize_started_events.read() {
        match territory_tabs_current_state.get() {
            TerritoryTabsState::Natural => territory_tabs_next_state.set(TerritoryTabsState::ResizingTerritories),
            _ => {warn!("[MAIN STATE] Invalid transition: {:?} -> ResizingTerritories", territory_tabs_current_state.get());}
        }
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

// See if the mouse has triggered any events for placeholders.

// Check for the cursor leaving the window.
// Despawn any TabMove and SpawnTerritory placeholders.
pub fn check_placeholder_types_leaving_window (
    mut commands: Commands,
    territory_tabs_current_state: Res<State<TerritoryTabsState>>,
    mut mouse_left_window_events: EventReader<CursorLeft>,
    mut placeholder_query: Query<(Entity, &mut Placeholder)>
) {
    for event in mouse_left_window_events.read() {
        match territory_tabs_current_state.get() {
            TerritoryTabsState::MovingTabs => {
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
            },
            _ => {}
        }
    }
}

// Check for the cursor entering a new window.
// Remove any SpawnWindow type placeholders.
pub fn check_placeholder_types_entering_window (
    mut commands: Commands,
    territory_tabs_current_state: Res<State<TerritoryTabsState>>,
    mut mouse_entered_window_events: EventReader<CursorEntered>,
    mut placeholder_query: Query<(Entity, &mut Placeholder)>
) {
    for event in mouse_entered_window_events.read() {
        match territory_tabs_current_state.get() {
            TerritoryTabsState::MovingTabs => {
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
            },
            _ => {}
        }
    }
}

// Check for mouse movement in the Window we're in.
// If so, see if we're in a Territory and change placeholder type.
pub fn check_placeholder_types_mouse_moving (
    mut commands: Commands,
    mouse_location_resource: Res<WorldMousePosition>,
    mut mouse_moved_in_window_events: EventReader<CursorMoved>,
    mut placeholder_query: Query<(Entity, &mut Placeholder)>
) {
    for event in mouse_moved_in_window_events.read() {
        for (placeholder_entity, mut placeholder) in &mut placeholder_query {
            match placeholder.placeholder_type {
                PlaceholderType::SpawnTerritory => {
                    if let Some(territory_entity) = mouse_location_resource.territory {
                        placeholder.placeholder_type = PlaceholderType::TabMove;
                        debug!("[CURSOR MOVED] Changed placeholder type from SpawnTerritory to TabMove!");
                    }
                },
                PlaceholderType::TabMove => {
                    if mouse_location_resource.territory == None {
                        placeholder.placeholder_type = PlaceholderType::SpawnTerritory;
                        debug!("[CURSOR MOVED] Changed placeholder type from TabMove to SpawnTerritory!");
                    }
                },
                PlaceholderType::SpawnWindow => {
                    warn!("[CURSOR MOVED] SpawnWindow type placeholder found inside of a Window??");
                    commands.entity(placeholder_entity).despawn();
                },
                PlaceholderType::TabOrigin => {},
                _ => {warn!("[CURSOR MOVED] Unusual placeholder type found!");}
            };
        }
    }
}

// With any non-Natural states ongoing, check for mouse movement.
// Calculate the visual_rects of the placeholders and determine spawn validity.
// Subject to on_event run condition, only runs when not in the Natural state.
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
            // Get the event window's size for later.
            let window_rect = Rect::from_center_size(
                Vec2::new(0.0, 0.0),
                Vec2::new(window.width(), window.height())
            );
            for mut placeholder in &mut placeholder_query {
                match placeholder.placeholder_type {
                    PlaceholderType::SpawnTerritory => {
                        // Get upper left coord. Adjust slightly for tab_offsets.
                        let worldspace_upper_left = Vec2::new(
                            mouse_location_resource.worldspace_pos.x - territory_settings.inner_margins.x,
                            mouse_location_resource.worldspace_pos.y + territory_settings.inner_margins.y
                        );

                        // Get the initial minimum and default territory rects.
                        let mut proposed_worldspace_rects = vec![
                            Rect::from_corners(
                                worldspace_upper_left, 
                                Vec2::new(
                                    worldspace_upper_left.x + territory_settings.min_size.x,
                                    worldspace_upper_left.y - territory_settings.min_size.y
                                )
                            ),
                            Rect::from_corners(
                                worldspace_upper_left, 
                                Vec2::new(
                                    worldspace_upper_left.x + territory_settings.default_size.x,
                                    worldspace_upper_left.y - territory_settings.default_size.y
                                )
                            )];

                        // Clip off anything outside the window.
                        proposed_worldspace_rects[1] = window_rect.intersect(proposed_worldspace_rects[1]);

                        // Intersecting territories clip off pieces of our initial default rect too.
                        for (parent, territory) in &territory_query {
                            let territory_conflict = proposed_worldspace_rects[1].intersect(territory.worldspace_rect());
                            let territory_window = parent.get();
                            if territory_window == event.window && !territory_conflict.is_empty() {
                            
                                let conflict_angle = (worldspace_upper_left.y - territory.worldspace_rect().center().y)
                                    .atan2(worldspace_upper_left.x - territory.worldspace_rect().center().x);

                                if conflict_angle <= FRAC_PI_4 && conflict_angle >= -FRAC_PI_4 {
                                    proposed_worldspace_rects[1].min.x += territory_conflict.width();
                                } 
                                else if conflict_angle >= FRAC_PI_4 && conflict_angle <= 3.0 * FRAC_PI_4 {
                                    proposed_worldspace_rects[1].min.y += territory_conflict.height();
                                }
                                else if (conflict_angle >= 3.0 * FRAC_PI_4 && conflict_angle <= PI)
                                    || (conflict_angle >= -PI && conflict_angle <= -3.0 * FRAC_PI_4) {
                                    proposed_worldspace_rects[1].max.x -= territory_conflict.width();
                                }
                                else if conflict_angle >= -3.0 * FRAC_PI_4 && conflict_angle <= -FRAC_PI_4 {
                                    proposed_worldspace_rects[1].max.y -= territory_conflict.height();
                                }
                                else{
                                    warn!{"Unusual conflict angle found during placeholder calculations!"}
                                }
                            }
                        }
                        // If the minimum still fits inside the clipped default, we're good to spawn.
                        // If not, ignore this frame's data to keep the last valid data.
                        if proposed_worldspace_rects[1].contains(proposed_worldspace_rects[0].min) 
                        && proposed_worldspace_rects[1].contains(proposed_worldspace_rects[0].max) {
                            placeholder.worldspace_visual_rects = proposed_worldspace_rects;
                            placeholder.world_to_screen(window.width(), window.height());
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
    for (entity, placeholder_parent, placeholder) in & placeholders_query {
        match placeholder.placeholder_type {
            PlaceholderType::SpawnTerritory => {
                if let Some(territory_parent) = placeholder_parent {
                    if placeholder.valid_spawn {
                        if let Some(mouse_window) = mouse_location_resource.window { 
                            let new_territory_id = commands.spawn((
                                EntityName("[TERRITORY] Spawned By Placeholder".to_string()),
                                CleanupOnWindowClose,
                                Territory::new(
                                    placeholder.screenspace_visual_rects[1],
                                    placeholder.worldspace_visual_rects[1],
                                    Rect::new(0.25, 0.25, 0.75, 0.75)
                                ),
                                SpatialBundle::default(),
                                DisplayLibrary::BevyEgui,
                            ))  .id();
                            commands.entity(mouse_window).add_child(new_territory_id);
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

// On the start of a territory drag, 
// Find the **worldspace** mouse position relative to `Territory.worldspace_rect.center()`
pub fn find_mouse_territory_interact_pos (
    mut mouse_location_resource: ResMut<WorldMousePosition>,
    window_query: Query<&Window>,
    territory_query: Query<&Territory>
) {
    if let Some(window_entity) = mouse_location_resource.window {
        if let Ok(window) = window_query.get(window_entity) {
            if let Some(territory_entity) = mouse_location_resource.territory {
                if let Ok(territory) = territory_query.get(territory_entity) {
                    let worldspace_interaction_pos = mouse_location_resource.worldspace_pos
                        .sub(territory.worldspace_rect().center());
                    mouse_location_resource.interaction_pos = worldspace_interaction_pos;
                    return;
                }
                warn!("[TERRITORY DRAG START] Unable to get territory from query!");
            }
            warn!("[TERRITORY DRAG START] Mouse resource had no territory!");
        }
        warn!("[TERRITORY DRAG START] Unable to get window from query!");
    }
    warn!("[TERRITORY DRAG START] Mouse resource had no window!");
}

// Examines a territory dragged event to determine move validity and applys the movement if it's good.
// Does nothing if the move is invalid.
pub fn apply_validated_territory_drag_delta (
    mut territory_dragged_events: EventReader<TerritoryDragged>,
    window_query: Query<&Window>,
    mut territory_paramset: ParamSet<(
        Query<(Entity, &Territory)>,
        Query<&mut Territory>
    )>
) {
    for drag_event in territory_dragged_events.read() {

        if let Ok((dragged_territory_entity, dragged_territory)) 
        = territory_paramset.p0().get(drag_event.territory_entity) {

            let proposed_rect = Rect::from_center_size(
                Vec2::new(
                    dragged_territory.worldspace_rect().center().x + drag_event.mouse_delta.x,
                    dragged_territory.worldspace_rect().center().y + drag_event.mouse_delta.y
                ), 
                dragged_territory.worldspace_rect().size()
            );

            // Check if the dragged Territory is outside the window. Exit early if so.
            if let Ok(window) = window_query.get(drag_event.window_entity) {
                if !dragged_territory.is_inside_worldspace_window(window.width(), window.height()) {
                   return 
                }
            }
            else {return}

            // Check for conflicts with other Territories and exit early if one is found.
            for (territory_entity, territory) in & territory_paramset.p0() {
                if territory_entity != dragged_territory_entity {
                    if !proposed_rect.intersect(territory.worldspace_rect()).is_empty() {return}                    
                }
            }
        }
        else {return}

        // Passed all checks without exiting, apply the delta
        if let Ok(mut dragged_territory) = territory_paramset.p1().get_mut(drag_event.territory_entity) {
            if let Ok(window) = window_query.get(drag_event.window_entity) {    
                dragged_territory
                    .move_worldspace_pos(drag_event.mouse_delta.x, drag_event.mouse_delta.y)
                    .world_to_screen(window.width(), window.height());
            }
        }
    }
}

// Read Territory drag event and update territory position.
pub fn determine_territory_drag_position (
    mut territory_dragged_events: EventReader<TerritoryDragged>,
    window_query: Query<(Entity, &Window)>,
    mut territory_query: Query<(Entity, &mut Territory)>
) {
    for event in territory_dragged_events.read() {
        for (window_entity, window) in & window_query {
            for (territory_entity, mut territory) in &mut territory_query {
                if territory_entity == event.territory_entity && event.window_entity == window_entity{
                    let new_rect = Rect::from_center_size(
                        territory.worldspace_rect().center().add(event.mouse_delta),
                        territory.worldspace_rect().size()
                        );
                    territory
                        .set_worldspace_rect(new_rect)
                        .world_to_screen(window.width(), window.height());
                }
            }
        }
    }
}



// Detect Territory collisions during Territory drag state and update position.
// Window edge collision will be a separate system.
pub fn check_territory_drag_collision (
    mut territory_dragged_events: EventReader<TerritoryDragged>,
    window_query: Query<&Window>,
    mut territory_query: Query<(Entity, &mut Territory)>
) {
    for event in territory_dragged_events.read() {
        if let Ok(window) = window_query.get(event.window_entity) {
            let mut territory_combinations = territory_query
                .iter_combinations_mut();
            while let Some([
                (territory_a_entity, mut territory_a),
                (territory_b_entity, mut territory_b)
                ]) = territory_combinations.fetch_next() {

                if territory_a_entity == event.territory_entity {
                    territory_a
                        .apply_worldspace_drag_collision_with(territory_b.worldspace_rect())
                        .world_to_screen(window.width(), window.height());
                } 
                if territory_b_entity == event.territory_entity {
                    territory_b
                        .apply_worldspace_drag_collision_with(territory_a.worldspace_rect())
                        .world_to_screen(window.width(), window.height());
                } 
            }
        }
    }
}

// Detect Territory bumping up against the window edge. Don't let it out!
// Note that egui will constrain its windows inside our windows even without this.
// However, we want TerritoryTabs logic decoupled from any display libraries.
pub fn check_window_drag_collision (
    mut territory_dragged_events: EventReader<TerritoryDragged>,
    window_query: Query<&Window>,
    mut territory_query: Query<&mut Territory>
) {
    for event in territory_dragged_events.read() {
        if let Ok(window) = window_query.get(event.window_entity) {
            if let Ok(mut territory) = territory_query.get_mut(event.territory_entity) {
                let window_rect = Rect::from_center_size(
                    Vec2::ZERO, 
                    Vec2::new(window.width(),window.height())
                );
                if window_rect.contains(territory.worldspace_rect().min)
                && window_rect.contains(territory.worldspace_rect().max) {continue;}

                if territory.worldspace_rect().min.x < window_rect.min.x {
                    let delta_x = window_rect.min.x - territory.worldspace_rect().min.x;
                    territory.move_worldspace_pos(
                        delta_x,
                        0.0
                    );
                }
                if territory.worldspace_rect().min.y < window_rect.min.y {
                    let delta_y = window_rect.min.y - territory.worldspace_rect().min.y;
                    territory.move_worldspace_pos(
                        0.0,
                        delta_y
                    );
                }
                if territory.worldspace_rect().max.x > window_rect.max.x {
                    let delta_x = window_rect.max.x - territory.worldspace_rect().max.x;
                    territory.move_worldspace_pos(
                        delta_x,
                        0.0
                    );
                }
                if territory.worldspace_rect().max.y > window_rect.max.y {
                    let delta_y = window_rect.max.y - territory.worldspace_rect().max.y;
                    territory.move_worldspace_pos(
                        0.0,
                        delta_y
                    );
                }
                territory.world_to_screen(window.width(), window.height());
            }
        }
    }
}

// A Territory is resizing! Calculate all of the window and Territory interactions with this resizing event.
// This first part restricts the resize from invalid moves, like resizing into a locked Territory.
pub fn determine_territory_resize_boundaries (
    mut gizmos: Gizmos,
    territory_settings: Res<TerritorySettings>,
    mut territory_resizing_events: EventReader<TerritoryResizing>,
    window_query: Query<&Window>,
    mut territory_query: Query<(Entity, &mut Territory)>
) {
    for resize_event in territory_resizing_events.read() {
        if let Ok(window) = window_query.get(resize_event.window_entity) {

            // This system centers around pushing this proposed resizing rect out of invalid spaces.
            let mut proposed_resize_rect = Rect::new(0.0, 0.0, 0.0, 0.0);

            if let Ok((
                resizing_territory_entity, 
                resizing_territory
                )) = territory_query.get(resize_event.territory_entity) {
                
                // First, create the proposed resize rect from the delta. Using worldspace for everything if possible.
                proposed_resize_rect = Rect::from_center_size(
                    resizing_territory.worldspace_rect().center(),
                    Vec2::new(
                        resizing_territory.worldspace_rect().size().x + resize_event.delta_size.x, 
                        resizing_territory.worldspace_rect().size().y + resize_event.delta_size.y
                    )
                );

                // Before anything else, clip any of the proposed rect that falls outside the window area.
                let window_rect = Rect::from_center_size(
                    Vec2::ZERO, 
                    Vec2::new(window.width(),window.height())
                );

                // Left
                if proposed_resize_rect.min.x < window_rect.min.x {
                    let delta_min_x = window_rect.min.x - proposed_resize_rect.min.x;
                    proposed_resize_rect.min.x += delta_min_x;
                }
                // Down
                if proposed_resize_rect.min.y < window_rect.min.y {
                    let delta_min_y = window_rect.min.y - proposed_resize_rect.min.y;
                    proposed_resize_rect.min.y += delta_min_y;
                }
                // Right
                if proposed_resize_rect.max.x > window_rect.max.x {
                    let delta_max_x = window_rect.max.x - proposed_resize_rect.max.x;
                    proposed_resize_rect.max.x += delta_max_x;
                }
                // Top
                if proposed_resize_rect.max.y > window_rect.max.y {
                    let delta_max_y = window_rect.max.y - proposed_resize_rect.max.y;
                    proposed_resize_rect.max.y += delta_max_y;
                }

                // Begin the check iteration. Pare back the proposed rect if it makes any invalid moves.
                // Invalid moves like resizing into a locked Territory or pushing another below the minimum size.
                // Only the proposed rect is modified here.
                for (other_territory_entity, other_territory) in & territory_query {
                    if other_territory_entity == resizing_territory_entity {continue;}

                    let conflict_rect = resizing_territory.worldspace_rect()
                        .intersect(other_territory.worldspace_rect());
                    if conflict_rect.is_empty() {continue;}

                    // Find the conflict_rect's sector, which determines what direction we're resizing in.
                    // Reduces checking & comparing down to one dimension and direction, even when resizing corners!
                    let conflict_angle = (
                        resizing_territory.worldspace_rect().center().y - conflict_rect.center().y)
                        .atan2(
                        resizing_territory.worldspace_rect().center().x - conflict_rect.center().x);

                    gizmos.rect_2d(
                        conflict_rect.center(), 
                        0.0, 
                        conflict_rect.size(), 
                        Color::RED
                    );

                    // Right
                    if conflict_angle <= FRAC_PI_4 && conflict_angle >= -FRAC_PI_4 {
                        if other_territory.is_locked() {
                            proposed_resize_rect.max.x -= conflict_rect.width();
                            continue;
                        }
                        let distance_from_min = other_territory.worldspace_rect().width()
                            - conflict_rect.width() 
                            - territory_settings.min_size.x;
                        if distance_from_min < 0.0 {
                            println!("Right Side Minimum Violation, paring back proposed rect by {}", distance_from_min.abs());
                            proposed_resize_rect.max.x -= distance_from_min.abs();
                        }
                    } 
                    // Top
                    else if conflict_angle >= FRAC_PI_4 && conflict_angle <= 3.0 * FRAC_PI_4 {
                        if other_territory.is_locked() {
                            proposed_resize_rect.max.y -= conflict_rect.height();
                            continue;
                        }
                        let distance_from_min = other_territory.worldspace_rect().height()
                            - conflict_rect.height() 
                            - territory_settings.min_size.y;
                        if distance_from_min < 0.0 {
                            proposed_resize_rect.max.y -= distance_from_min.abs();
                        }
                    }
                    // Left
                    else if (conflict_angle >= 3.0 * FRAC_PI_4 && conflict_angle <= PI)
                        || (conflict_angle >= -PI && conflict_angle <= -3.0 * FRAC_PI_4) {
                        if other_territory.is_locked() {
                            proposed_resize_rect.min.x += conflict_rect.width();
                            continue;
                        }
                        let distance_from_min = other_territory.worldspace_rect().width()
                            - conflict_rect.width() 
                            - territory_settings.min_size.x;
                        if distance_from_min < 0.0 {
                            proposed_resize_rect.min.x += distance_from_min.abs();
                        }
                    }
                    // Down
                    else if conflict_angle >= -3.0 * FRAC_PI_4 && conflict_angle <= -FRAC_PI_4 {
                        if other_territory.is_locked() {
                            proposed_resize_rect.min.y += conflict_rect.height();
                            continue;
                        }
                        let distance_from_min = other_territory.worldspace_rect().height()
                            - conflict_rect.height() 
                            - territory_settings.min_size.y;
                        if distance_from_min < 0.0 {
                            proposed_resize_rect.min.y += distance_from_min.abs();
                        }
                    }
                }
            }
            if let Ok((
                _, 
                mut resizing_territory
                )) = territory_query.get_mut(resize_event.territory_entity) { 
                // Apply final proposed rect to the resizing territory before continuing to next resize system.
                // We need to mutably iter through the Territories again, so a separate system is borrow-checker friendly.
                
                resizing_territory
                .set_worldspace_rect(proposed_resize_rect)
                .world_to_screen(window.width(), window.height());
            }
        }
    }
}

// A Territory is resizing! Calculate all of the window and Territory interactions with this resizing event.
// This second part, now having a valid resize rect, resizes away all encroaching Territories.
pub fn apply_valid_territory_resize (
    mut territory_resizing_events: EventReader<TerritoryResizing>,
    window_query: Query<&Window>,
    mut territory_query: Query<(Entity, &mut Territory)>
) {
    for resize_event in territory_resizing_events.read() {
        if let Ok(window) = window_query.get(resize_event.window_entity) {

            // Begin the second iteration. The proposed rect is no longer possibly demanding invalid space.
            // So now we check again, but this time everyone else moves if there's a conflict.
            let mut territory_combinations = territory_query
                .iter_combinations_mut();
            while let Some([
                (territory_a_entity, territory_a),
                (territory_b_entity, territory_b)
                ]) = territory_combinations.fetch_next() {

                let resizing_territory;
                let mut adjusted_territory;

                if territory_a_entity == resize_event.territory_entity {
                    resizing_territory = territory_a;
                    adjusted_territory = territory_b;
                }
                else if territory_b_entity == resize_event.territory_entity {
                    adjusted_territory = territory_a;
                    resizing_territory = territory_b;
                }
                else {continue;}

                let conflict_rect = resizing_territory.worldspace_rect()
                    .intersect(adjusted_territory.worldspace_rect());
                if conflict_rect.is_empty() {continue;}

                let conflict_angle = (
                    resizing_territory.worldspace_rect().center().y - conflict_rect.center().y)
                    .atan2(
                    resizing_territory.worldspace_rect().center().x - conflict_rect.center().x);

                // Right
                if conflict_angle <= FRAC_PI_4 && conflict_angle >= -FRAC_PI_4 {
                    adjusted_territory.worldspace_rect().min.x += conflict_rect.width();
                } 
                // Top
                else if conflict_angle >= FRAC_PI_4 && conflict_angle <= 3.0 * FRAC_PI_4 {
                    adjusted_territory.worldspace_rect().min.y += conflict_rect.height();
                }
                // Left
                else if (conflict_angle >= 3.0 * FRAC_PI_4 && conflict_angle <= PI)
                    || (conflict_angle >= -PI && conflict_angle <= -3.0 * FRAC_PI_4) {
                    adjusted_territory.worldspace_rect().max.x -= conflict_rect.width();
                }
                // Down
                else if conflict_angle >= -3.0 * FRAC_PI_4 && conflict_angle <= -FRAC_PI_4 {
                    adjusted_territory.worldspace_rect().max.y -= conflict_rect.height();
                }   
                adjusted_territory.world_to_screen(window.width(), window.height());
            }
        }
    }
}