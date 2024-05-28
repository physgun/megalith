use bevy::prelude::*;
use bevy::window::*;
use bevy::render::camera::*;

use crate::components_ui::*;
use crate::events_ui::*;
use crate::input_manager::*;
use crate::resources_ui::*;
use crate::systems_common::*;

use std::f32::consts::FRAC_PI_4;
use std::f32::consts::PI;

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
    windows_query: Query<&Window>,
    territories_query: Query<(Entity, &Parent, &Territory)>,
    // TODO: Tab query here later!
) {
    // Reset mouse info so we don't keep around old data.
    // TODO: Move mouse info from resource to events
    mouse_location_resource.window = None;
    mouse_location_resource.territory = None;
    mouse_location_resource.tab = None;

    for (camera, camera_transform) in & cameras_query {    
        for window in & windows_query {
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
    mut territory_move_tab_exit_events: EventReader<TestChordJustReleased>
) {
    for event in territory_move_tab_exit_events.read() {
        match territory_tabs_current_state.get() {
            TerritoryTabsState::MovingTabs => territory_tabs_next_state.set(TerritoryTabsState::Natural),
            _ => {warn!("[MAIN STATE] Invalid transition: {:?} -> Natural", territory_tabs_current_state.get());}
        }
    }
}

// TerritoryTabs operation state machine handling enter events after.
pub fn territory_tabs_main_state_enter (
    territory_tabs_current_state: Res<State<TerritoryTabsState>>,
    mut territory_tabs_next_state: ResMut<NextState<TerritoryTabsState>>,
    mut territory_move_tab_enter_events: EventReader<TestChordJustPressed>
) {
    for event in territory_move_tab_enter_events.read() {
        match territory_tabs_current_state.get() {
            TerritoryTabsState::Natural => territory_tabs_next_state.set(TerritoryTabsState::MovingTabs),
            _ => {warn!("[MAIN STATE] Invalid transition: {:?} -> MovingTabs", territory_tabs_current_state.get());}
        }
    }
}









// Begin Territory motion refactor!

/// Check all [`Territory`]s who have a [`MoveRequest`] component and see what kind of movement they want.
/// Any [`Locked`] [`Territory`]s will have their [`MoveRequest`] component removed.
pub fn territory_move_eval_type (
    mut commands: Commands,
    window_query: Query<(&Window, &Children), With<TerritoryTabs>>,
    mut moving_territories_query: Query<(Entity, &Territory, Option<&Locked>, &mut MoveRequest)>
) {
    for (window, window_children) in &window_query {
        let mut moving_territories = moving_territories_query
            .iter_many_mut(window_children);
        while let Some(
            (territory_entity, territory, territory_locked, mut move_request)
        ) = moving_territories.fetch_next() {

            // A Locked Territory won't process any MoveRequest.
            if let Some(_locked) = territory_locked {
                commands.entity(territory_entity).remove::<MoveRequest>();
                continue;
            }

            if !move_request.proposed_worldspace_rect().is_empty() && move_request.proposed_screenspace_rect().is_empty() {
                debug!("Translated MoveRequest world rect to screen!");
                move_request.world_to_screen(window.width(), window.height());
            }
            
            if !move_request.proposed_screenspace_rect().is_empty() && move_request.proposed_worldspace_rect().is_empty() {
                debug!("Translated MoveRequest screen rect to world!");
                move_request.screen_to_world(window.width(), window.height());
            }

            match move_request.move_type() {
                MoveRequestType::Unknown => {

                    if move_request.proposed_worldspace_rect() == territory.worldspace_rect() {
                        commands.entity(territory_entity).remove::<MoveRequest>();
                        //debug!("MoveRequest found with identical rect to existing rect, and was removed!");
                        continue;
                    }

                    if territory.worldspace_rect().height() == move_request.proposed_worldspace_rect().height()
                    && territory.worldspace_rect().width() == move_request.proposed_worldspace_rect().width() {
                        move_request.move_type_drag();
                        //debug!("MoveRequest type changed to Drag!");
                    }
                    else {
                        move_request.move_type_drag();
                        debug!("MoveRequest type changed to `Resize`!");
                    }
                },
                MoveRequestType::Drag | MoveRequestType::Resize => {continue}
            };
        }
    }
}

/// Process all [`Territory`] & [`MoveRequest`] interactions with the window edge.
/// Clip off resizing proposals, move away dragging proposals.
pub fn territory_move_process_fringe (
    mut commands: Commands,
    window_query: Query<(Entity, &Window), With<TerritoryTabs>>,
    mut moving_territories_query: Query<(Entity, &Parent, &mut MoveRequest), With<Territory>>
) {
    for (window_entity, window) in & window_query {
        for (
            territory_entity,
            territory_parent,
            mut move_request
        ) in &mut moving_territories_query {
            if territory_parent.get() == window_entity {

                let window_rect = Rect::from_center_size(
                    Vec2::ZERO, 
                    Vec2::new(window.width(),window.height())
                );

                match move_request.move_type() {
                    MoveRequestType::Unknown => {
                        warn!("Unknown-type MoveRequest found on Territory during processing!");
                        commands.entity(territory_entity).remove::<MoveRequest>(); // Get outta here!
                    },
                    MoveRequestType::Drag => {
                        if window_rect.contains(move_request.proposed_worldspace_rect().min)
                        && window_rect.contains(move_request.proposed_worldspace_rect().max) {continue;}
        
                        if move_request.proposed_worldspace_rect().min.x < window_rect.min.x {
                            let delta_x = window_rect.min.x - move_request.proposed_worldspace_rect().min.x;
                            move_request.move_worldspace_delta(
                                Vec2::new(delta_x, 0.0),
                                Vec2::new(delta_x, 0.0),
                                window.width(),
                                window.height()
                            );
                        }
                        if move_request.proposed_worldspace_rect().min.y < window_rect.min.y {
                            let delta_y = window_rect.min.y - move_request.proposed_worldspace_rect().min.y;
                            move_request.move_worldspace_delta(
                                Vec2::new(0.0, delta_y),
                                Vec2::new(0.0, delta_y),
                                window.width(),
                                window.height()
                            );
                        }
                        if move_request.proposed_worldspace_rect().max.x > window_rect.max.x {
                            let delta_x = window_rect.max.x - move_request.proposed_worldspace_rect().max.x;
                            move_request.move_worldspace_delta(
                                Vec2::new(delta_x, 0.0),
                                Vec2::new(delta_x, 0.0),
                                window.width(),
                                window.height()
                            );
                        }
                        if move_request.proposed_worldspace_rect().max.y > window_rect.max.y {
                            let delta_y = window_rect.max.y - move_request.proposed_worldspace_rect().max.y;
                            move_request.move_worldspace_delta(
                                Vec2::new(0.0, delta_y),
                                Vec2::new(0.0, delta_y),
                                window.width(),
                                window.height()
                            );
                        }
                    },
                    MoveRequestType::Resize => {
                        let inbounds_rect = window_rect.intersect(move_request.proposed_worldspace_rect());

                        move_request.set_proposed_worldspace_rect(
                            inbounds_rect, 
                            window.width(), 
                            window.height()
                        );
                    }
                }
            }
        }
    }
}

/// For all entities with [`Territory`] and a [`MoveRequest`], iterate through all conflicting [`Territory`]s.
/// If we're resizing, see how much we can push away others. If dragging, move away from others per atan2 results.
/// If there's still a conflict at the end, remove the [`MoveRequest`].
pub fn territory_move_check_others (
    mut commands: Commands,
    territory_settings: Res<TerritorySettings>,
    window_query: Query<(Entity, &Window), With<TerritoryTabs>>,
    mut moving_territories_query: Query<(Entity, &Parent, &Territory, &mut MoveRequest)>,
    mut other_territories_query: Query<(Entity, &Parent, &mut Territory, Option<&Locked>), (Without<MoveRequest>, Without<Overlay>)>
) {
    for (window_entity, window) in & window_query {
        for (
            territory_entity,
            territory_parent,
            territory,
            mut move_request
        ) in &mut moving_territories_query {
            if territory_parent.get() == window_entity {

                match move_request.move_type() {
                    MoveRequestType::Unknown => {
                        warn!("Unknown-type MoveRequest found on Territory during processing!");
                        commands.entity(territory_entity).remove::<MoveRequest>();
                    },
                    MoveRequestType::Drag => {
                        for (
                            other_entity, 
                            other_parent, 
                            mut other_territory,
                            territory_locked
                        ) in &mut other_territories_query {
                            if other_parent.get() == window_entity {

                                let conflict_rect = move_request.proposed_worldspace_rect()
                                    .intersect(other_territory.worldspace_rect());
                                if conflict_rect.is_empty() {continue;}

                                // If the user goes nuts, they can drag Territories fast enough that the conflict rect
                                // is entirely contained inside our Territory rect. Remaining space handles that case. Mostly.
                                // TODO: Handle that case better than mostly.
                                if conflict_rect.height() >= conflict_rect.width() {

                                    if move_request.proposed_worldspace_rect().center().x 
                                    >= other_territory.worldspace_rect().center().x {
                                        let remaining_space = other_territory.worldspace_rect().max.x - conflict_rect.max.x;
                                        move_request.move_worldspace_delta(
                                            Vec2::new(conflict_rect.width() + remaining_space, 0.0),
                                            Vec2::new(conflict_rect.width() + remaining_space, 0.0),
                                            window.width(),
                                            window.height()
                                        );
                                    }
                                    else {
                                        let remaining_space = conflict_rect.min.x - other_territory.worldspace_rect().min.x;
                                        move_request.move_worldspace_delta(
                                            Vec2::new(-1.0 * conflict_rect.width() - remaining_space, 0.0),
                                            Vec2::new(-1.0 * conflict_rect.width() - remaining_space, 0.0),
                                            window.width(),
                                            window.height()
                                        );
                                    }
                                }
                                else {

                                    if move_request.proposed_worldspace_rect().center().y 
                                    >= other_territory.worldspace_rect().center().y {
                                        let remaining_space = other_territory.worldspace_rect().max.y - conflict_rect.max.y;
                                        move_request.move_worldspace_delta(
                                            Vec2::new(0.0, conflict_rect.height() + remaining_space),
                                            Vec2::new(0.0, conflict_rect.height() + remaining_space),
                                            window.width(),
                                            window.height()
                                        );
                                    }
                                    else {
                                        let remaining_space = conflict_rect.min.y - other_territory.worldspace_rect().min.y;
                                        move_request.move_worldspace_delta(
                                            Vec2::new(0.0, -1.0 * conflict_rect.height() - remaining_space),
                                            Vec2::new(0.0, -1.0 * conflict_rect.height() - remaining_space),
                                            window.width(),
                                            window.height()
                                        );
                                    } 
                                }

                            }
                        }
                        // Swing through again and verify no conflicts remain. If there are conflicts, remove MoveRequest.
                        for (
                            other_entity, 
                            other_parent, 
                            mut other_territory,
                            territory_locked
                        ) in &mut other_territories_query {
                            if other_parent.get() == window_entity {

                                let conflict_rect = move_request.proposed_worldspace_rect()
                                    .intersect(other_territory.worldspace_rect());
                                if !conflict_rect.is_empty() {
                                    warn!("Drag-type MoveRequest still found conflicts after processing. MoveRequest removed!");
                                    commands.entity(territory_entity).remove::<MoveRequest>();
                                }
                            }
                        }
                    },
                    MoveRequestType::Resize => {
                        for (
                            other_entity, 
                            other_parent, 
                            mut other_territory,
                            territory_locked
                        ) in &mut other_territories_query {
                            if other_parent.get() == window_entity {
                                
                                let conflict_rect = move_request.proposed_worldspace_rect()
                                    .intersect(other_territory.worldspace_rect());
                                if conflict_rect.is_empty() {continue;}

                                // Find the conflict_rect's sector, which determines what direction we pared back proposed resize.
                                let conflict_angle = (
                                    move_request.proposed_worldspace_rect().center().y - conflict_rect.center().y)
                                    .atan2(
                                    move_request.proposed_worldspace_rect().center().x - conflict_rect.center().x);

                                // Cycle through and see first how far we can move our resize, paring back as necessary.
                                // Don't move other Territories yet.

                                // Right
                                if conflict_angle <= FRAC_PI_4 && conflict_angle >= -FRAC_PI_4 {
                                    if let Some(locked) = territory_locked {
                                        move_request.move_worldspace_delta(
                                            Vec2::ZERO, 
                                            Vec2::new(-1.0 * conflict_rect.width(), 0.0), 
                                            window.width(), 
                                            window.height()
                                        );
                                        continue;
                                    }

                                    let conflict_overreach = conflict_rect.width()
                                     - (other_territory.worldspace_rect().width() - territory_settings.min_size.x);

                                    if conflict_overreach > 0.0 {
                                        move_request.move_worldspace_delta(
                                            Vec2::ZERO, 
                                            Vec2::new(-1.0 * conflict_overreach, 0.0), 
                                            window.width(), 
                                            window.height()
                                        );
                                    }
                                } 
                                // Top
                                else if conflict_angle >= FRAC_PI_4 && conflict_angle <= 3.0 * FRAC_PI_4 {
                                    if let Some(locked) = territory_locked {
                                        move_request.move_worldspace_delta(
                                            Vec2::ZERO, 
                                            Vec2::new(0.0, -1.0 * conflict_rect.height()), 
                                            window.width(), 
                                            window.height()
                                        );
                                        continue;
                                    }

                                    let conflict_overreach = conflict_rect.height()
                                     - (other_territory.worldspace_rect().height() - territory_settings.min_size.y);

                                    if conflict_overreach > 0.0 {
                                        move_request.move_worldspace_delta(
                                            Vec2::ZERO, 
                                            Vec2::new(0.0, -1.0 * conflict_overreach), 
                                            window.width(), 
                                            window.height()
                                        );
                                    }
                                }
                                // Left (atan2 is discontinuous at PI, as its range is -PI to PI)
                                else if (conflict_angle >= 3.0 * FRAC_PI_4 && conflict_angle <= PI)
                                    || (conflict_angle >= -PI && conflict_angle <= -3.0 * FRAC_PI_4) {
                                    if let Some(locked) = territory_locked {
                                        move_request.move_worldspace_delta(
                                            Vec2::new(1.0 * conflict_rect.width(), 0.0), 
                                            Vec2::ZERO, 
                                            window.width(), 
                                            window.height()
                                        );
                                        continue;
                                    }

                                    let conflict_overreach = conflict_rect.width()
                                        - (other_territory.worldspace_rect().width() - territory_settings.min_size.x);

                                    if conflict_overreach > 0.0 {
                                        move_request.move_worldspace_delta(
                                            Vec2::new(1.0 * conflict_overreach, 0.0),
                                            Vec2::ZERO, 
                                            window.width(), 
                                            window.height()
                                        );
                                    }
                                }
                                // Down
                                else if conflict_angle >= -3.0 * FRAC_PI_4 && conflict_angle <= -FRAC_PI_4 {
                                    if let Some(locked) = territory_locked {
                                        move_request.move_worldspace_delta(
                                            Vec2::new(0.0, 1.0 * conflict_rect.height()), 
                                            Vec2::ZERO, 
                                            window.width(), 
                                            window.height()
                                        );
                                        continue;
                                    }

                                    let conflict_overreach = conflict_rect.height()
                                     - (other_territory.worldspace_rect().height() - territory_settings.min_size.y);

                                    if conflict_overreach > 0.0 {
                                        move_request.move_worldspace_delta(
                                            Vec2::new(0.0, 1.0 * conflict_overreach), 
                                            Vec2::ZERO, 
                                            window.width(), 
                                            window.height()
                                        );
                                    }
                                }
                            }
                        }

                        // Now that the MoveRequest knows what its final size can be, we push away other territories.
                        for (
                            other_entity, 
                            other_parent, 
                            mut other_territory,
                            territory_locked
                        ) in &mut other_territories_query {
                            if other_parent.get() == window_entity {

                                let conflict_rect = move_request.proposed_worldspace_rect()
                                    .intersect(other_territory.worldspace_rect());
                                if conflict_rect.is_empty() {continue;}

                                // Find the conflict_rect's sector, which determines what direction we resize the other Territory.
                                let conflict_angle = (
                                    move_request.proposed_worldspace_rect().center().y - conflict_rect.center().y)
                                    .atan2(
                                    move_request.proposed_worldspace_rect().center().x - conflict_rect.center().x);

                                // Second run-through to push other Territories out of our, now valid, resize MoveRequest.
                                // Don't forget to invert the direction of resize, 
                                // since the proposed resize's right is the other Territory's left.

                                // Right
                                if conflict_angle <= FRAC_PI_4 && conflict_angle >= -FRAC_PI_4 {
                                    other_territory.move_worldspace_corners(
                                        Vec2::new(1.0 * conflict_rect.width(), 0.0),
                                        Vec2::ZERO,
                                        window.width(),
                                        window.height()
                                    );
                                } 
                                // Top
                                else if conflict_angle >= FRAC_PI_4 && conflict_angle <= 3.0 * FRAC_PI_4 {
                                    other_territory.move_worldspace_corners(
                                        Vec2::new(0.0, 1.0 * conflict_rect.height()),
                                        Vec2::ZERO,
                                        window.width(),
                                        window.height()
                                    );
                                }
                                // Left (atan2 is discontinuous at PI, as its range is -PI to PI)
                                else if (conflict_angle >= 3.0 * FRAC_PI_4 && conflict_angle <= PI)
                                    || (conflict_angle >= -PI && conflict_angle <= -3.0 * FRAC_PI_4) {
                                    other_territory.move_worldspace_corners(
                                        Vec2::ZERO,
                                        Vec2::new(-1.0 * conflict_rect.height(), 0.0),
                                        window.width(),
                                        window.height()
                                    );
                                }
                                // Down
                                else if conflict_angle >= -3.0 * FRAC_PI_4 && conflict_angle <= -FRAC_PI_4 {
                                    other_territory.move_worldspace_corners(
                                        Vec2::ZERO,
                                        Vec2::new(0.0, -1.0 * conflict_rect.height()),
                                        window.width(),
                                        window.height()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// All [`MoveRequest`] processing done, now apply any surviving [`MoveRequest`]s.
pub fn territory_move_apply_proposed (
    mut commands: Commands,
    window_query: Query<(&Window, &Children), With<TerritoryTabs>>,
    mut moving_territories_query: Query<(Entity, &mut Territory, &MoveRequest)>
) {
    for (window, window_children) in &window_query {
        let mut move_requests = moving_territories_query.iter_many_mut(window_children);
        while let Some(
            (territory_entity, mut territory, move_request)
        ) = move_requests.fetch_next() {
            match move_request.move_type {
                MoveRequestType::Unknown => {
                    warn!("Unknown-type MoveRequest found on Territory during application!");
                    commands.entity(territory_entity).remove::<MoveRequest>();
                }
                MoveRequestType::Drag | MoveRequestType::Resize => {
                    //debug!("Applying {:?}", move_request.proposed_worldspace_rect());
                    territory.set_worldspace_rect(
                        move_request.proposed_worldspace_rect(), 
                        window.width(), 
                        window.height()
                    );
                    commands.entity(territory_entity).remove::<MoveRequest>();
                }
            }
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
                                Territory {
                                    screenspace_rect: placeholder.screenspace_visual_rects[1],
                                    worldspace_rect: placeholder.worldspace_visual_rects[1],
                                    ..Default::default()
                                },
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