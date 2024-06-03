//! Contains all Events, Systems, SystemSets, and Plugins pertaining to a [`Territory`].

use std::f32::consts::FRAC_PI_4;
use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::window::*;
use bevy::render::camera::*;

use crate::components_territory::*;
use crate::display_territory_sickle::*;
use crate::input_manager::*;


pub struct TerritoryPlugin;

impl Plugin for TerritoryPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TerritorySizeSettings>()
            .insert_state(TerritoryTabsMode::Operating)
            .add_event::<MoveRequestApplied>()
            .add_event::<TerritorySpawnRequest>()
            .add_systems(Update, (
                (
                    configure_os_window
                        .run_if(on_event::<WindowCreated>()),
                    configure_os_window_sickle
                        .run_if(on_event::<WindowCreated>())
                )
                    .chain()
                    .in_set(WindowConfig),
                (
                    spawn_territory_sickle,
                    display_debug_gizmos
                )
                    .chain()
                    .in_set(TerritoryDisplay),
                (

                    (
                        empty_if_no_territories
                            .run_if(territory_removed.or_else(territory_spawned))
                            .before(test_delete_all_territories),
                        test_delete_all_territories
                            .run_if(on_event::<RemoveTerritoriesKeyPressed>())
                    ) .in_set(TerritoryUpdateState),
                    (
                    territory_move_eval_type,
                    territory_move_process_fringe,
                    territory_move_check_others,
                    territory_move_apply_proposed
                    )
                        .chain()
                        .in_set(TerritoryUpdateMotion)
                        .run_if(any_with_component::<MoveRequest>)

                )
                    .in_set(TerritoryUpdate)
            ))
            .configure_sets(Update,
                (
                        WindowConfig.before(TerritoryDisplay),
                        TerritoryDisplay.before(TerritoryUpdate)
                ),
        );
    }
}

/// All display logic.
#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct TerritoryDisplay;

/// Contains systems for managing all entities with [`Window`] and [`TerritoryTabs`] components.
#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct WindowConfig;

/// Contains systems that render the [`Territory`] using the `bevy_sickle` library.
#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct TerritoryDisplaySickle;

/// All update logic.
#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct TerritoryUpdate;

/// Contains systems that act as state machines for [`TerritoryTabsMode`].
#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct TerritoryUpdateState;

/// Contains systems that act on a [`MoveRequest`].
#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct TerritoryUpdateMotion;

/// Sent when a UI element is issued a [`MoveRequest`] component.
#[derive(Event)]
pub struct MoveRequestApplied;

/// Sent when a system has commanded a [`Territory`] to spawn in a `Window` `Entity`.
#[derive(Event)]
pub struct TerritorySpawnRequest {
    pub window_entity: Entity,
    pub screenspace_rect: Rect,
    pub worldspace_rect: Rect,
    //pub relative_screenspace_rect: Rect,
    //pub relative_worldspace_rect: Rect,
    pub display_library: DisplayLibrary
}

/// Debug gizmos!
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

/// TODO: Refactor this out!
#[derive(Component)]
pub struct MouseSeekingCamera;

/// A default configuration for the OS windows. Background camera, names, etc.
/// Summoned by a [`WindowCreated`] event and configures that exact window.
pub fn configure_os_window(
    mut commands: Commands,
    mut window_spawn_detected_events: EventReader<WindowCreated>,
    mut window_query: Query<&mut Window>
) {
    for event in window_spawn_detected_events.read() {
        if let Ok(mut window) = window_query.get_mut(event.window) {
            window.title = "Territory Tabs".to_string();

            let child_camera = commands.spawn((
                Name::new("[CAMERA] Territory Tabs UI Camera"),
                Camera2dBundle {
                    camera: Camera {
                        clear_color: ClearColorConfig::Custom(Color::WHITE), 
                        target: RenderTarget::Window(WindowRef::Entity(event.window)),
                        ..Default::default() 
                        }, 
                    ..Default::default()
                },
                TerritoryTabsCamera,
                MouseSeekingCamera // TODO: Refactor this out.
            )).id();
    
            // Add camera as child to the window and give additional components.
            commands.entity(event.window)
                .add_child(child_camera)
                .insert((
                    Name::new("[WINDOW] Territory Tabs Window"),
                    TerritoryTabs,
                    DisplayLibrary::BevySickle,
                    SpatialBundle::default()
            ));
        }
    }
}

/// Run condition checking if a [`Territory`] spawned recently.
pub fn territory_spawned (
    added_query: Query<&Territory, Added<Territory>>
) -> bool {
    !added_query.is_empty()
}

/// Run condition checking if a [`Territory`] was removed recently.
pub fn territory_removed (
    removed_query: RemovedComponents<Territory>
) -> bool {
    !removed_query.is_empty()
}

/// When a [`Territory`] component is removed, check to see if there are any left.
/// Change [`TerritoryTabsMode`] state to [`TerritoryTabsMode::Empty`] if so.
/// Change it back when a new one is spawned.
pub fn empty_if_no_territories (
    territory_tabs_mode: Res<State<TerritoryTabsMode>>,
    mut set_territory_tabs_mode: ResMut<NextState<TerritoryTabsMode>>,
    territory_query: Query<&Territory>,
) {
    if territory_query.is_empty() {
        match territory_tabs_mode.get() {
            TerritoryTabsMode::Empty => { 
                warn!("Unexpected transition: Empty -> Empty"); 
            }
            TerritoryTabsMode::Operating => { 
                set_territory_tabs_mode.set(TerritoryTabsMode::Empty); 
            }
            TerritoryTabsMode::MovingTerritories => { 
                set_territory_tabs_mode.set(TerritoryTabsMode::Empty);
                warn!("Unexpected transition: MovingTerritories -> Empty"); 
            }
            TerritoryTabsMode::MovingTabs => { 
                set_territory_tabs_mode.set(TerritoryTabsMode::Empty);
                warn!("Unexpected transition: MovingTabs -> Empty"); 
            }
        }
    }
    else {
        match territory_tabs_mode.get() {
            TerritoryTabsMode::Empty => { set_territory_tabs_mode.set(TerritoryTabsMode::Operating); }
            _ => {}
        }
    }
}

/// Debug system Removes all entities with [`Territory`] when the dev key chord event is read..
pub fn test_delete_all_territories (
    mut commands: Commands,
    mut remove_territories_key_pressed: EventReader<RemoveTerritoriesKeyPressed>,
    window_query: Query<&Children, With<Window>>,
    territory_query: Query<Entity, With<Territory>>
) {
    for _event in remove_territories_key_pressed.read() {
        for window_children in & window_query {
            let mut territories = territory_query.iter_many(window_children);
            while let Some(territory_entity) =  territories.fetch_next(){
                commands.entity(territory_entity).despawn_recursive();
            }
        }
    }
}

/// Check all [`Territory`]s who have a [`MoveRequest`] component and see what kind of movement they want.
/// Any [`Locked`] [`Territory`]s will have their [`MoveRequest`] component removed.
pub fn territory_move_eval_type (
    mut commands: Commands,
    window_query: Query<(&Window, &Children), With<TerritoryTabs>>,
    mut moving_territories_query: Query<(Entity, &Territory, Option<&Locked>, &mut MoveRequest)>
) {
    for (window, window_children) in & window_query {
        let mut moving_territories = moving_territories_query.iter_many_mut(window_children);
        while let Some(
            (territory_entity, territory, territory_locked, mut move_request)
        ) = moving_territories.fetch_next() {

            // A Locked Territory won't process any MoveRequest.
            if let Some(_locked) = territory_locked {
                commands.entity(territory_entity).remove::<MoveRequest>();
                continue;
            }

            if !move_request.proposed_worldspace_rect().is_empty() && move_request.proposed_screenspace_rect().is_empty() {
                move_request.world_to_screen(window.width(), window.height());
            }
            
            if !move_request.proposed_screenspace_rect().is_empty() && move_request.proposed_worldspace_rect().is_empty() {
                move_request.screen_to_world(window.width(), window.height());
            }

            match move_request.move_type() {
                MoveRequestType::Unknown => {

                    if move_request.proposed_worldspace_rect() == territory.worldspace_rect() {
                        commands.entity(territory_entity).remove::<MoveRequest>();
                        debug!("MoveRequest found with identical rect to existing rect, and was removed!");
                        continue;
                    }

                    if territory.worldspace_rect().height() == move_request.proposed_worldspace_rect().height()
                    && territory.worldspace_rect().width() == move_request.proposed_worldspace_rect().width() {
                        move_request.move_type_drag();
                        //debug!("MoveRequest type changed to Drag!");
                    }
                    else {
                        move_request.move_type_resize();
                        //debug!("MoveRequest type changed to Resize!");
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
    window_query: Query<(&Window, &Children), With<TerritoryTabs>>,
    mut moving_territories_query: Query<(Entity, &mut MoveRequest), With<Territory>>
) {
    for (window, window_children) in & window_query {
        let mut moving_territories = moving_territories_query.iter_many_mut(window_children);
        while let Some((territory_entity, mut move_request)) = moving_territories.fetch_next() {
            
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

/// For all entities with [`Territory`] and a [`MoveRequest`], iterate through all conflicting [`Territory`]s.
/// If we're resizing, see how much we can push away others. If dragging, move away from others.
/// If there's still a conflict at the end, remove the [`MoveRequest`].
pub fn territory_move_check_others (
    mut commands: Commands,
    territory_settings: Res<TerritorySizeSettings>,
    window_query: Query<
        (&Window, &Children), 
        With<TerritoryTabs>
        >,
    mut moving_territories_query: Query<(Entity, &mut MoveRequest)>,
    mut other_territories_query: Query<
        (&mut Territory, Option<&Locked>), 
        Without<MoveRequest>
        >
) {
    for (window, window_children) in & window_query {
        let mut moving_territories = moving_territories_query.iter_many_mut(window_children);
        while let Some(
            (territory_entity, mut move_request)
        ) = moving_territories.fetch_next() {

            match move_request.move_type() {

                MoveRequestType::Unknown => {
                    warn!("Unknown-type MoveRequest found on Territory during processing!");
                    commands.entity(territory_entity).remove::<MoveRequest>();
                },

                MoveRequestType::Drag => {
                    let mut other_territories = other_territories_query
                        .iter_many_mut(window_children);
                    while let Some(
                        (other_territory, _is_locked)
                    ) = other_territories.fetch_next() {

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

                    // Swing through again and verify no conflicts remain. If there are conflicts, remove MoveRequest.
                    let mut other_territories = other_territories_query
                        .iter_many_mut(window_children);
                    while let Some(
                        (other_territory, _is_locked)
                    ) = other_territories.fetch_next() {

                        let conflict_rect = move_request.proposed_worldspace_rect()
                            .intersect(other_territory.worldspace_rect());
                        if !conflict_rect.is_empty() {
                            warn!("Drag-type MoveRequest still found conflicts after processing. MoveRequest removed!");
                            commands.entity(territory_entity).remove::<MoveRequest>();
                        }
                    }
                },

                MoveRequestType::Resize => {
                    let mut other_territories = other_territories_query
                        .iter_many_mut(window_children);
                    while let Some(
                        (other_territory, is_locked)
                    ) = other_territories.fetch_next() {
                            
                        let conflict_rect = move_request.proposed_worldspace_rect()
                            .intersect(other_territory.worldspace_rect());
                        if conflict_rect.is_empty() {continue;}

                        // Find the conflict_rect's sector, which determines what direction we pared back proposed resize.
                        let conflict_angle = (
                            move_request.proposed_worldspace_rect().center().y - conflict_rect.center().y)
                            .atan2(
                            move_request.proposed_worldspace_rect().center().x - conflict_rect.center().x);

                        // Cycle through and see, first, how far we can move our resize, paring back as necessary.
                        // Don't move away other Territories yet. Some might be locked!

                        // Right
                        if conflict_angle <= FRAC_PI_4 && conflict_angle >= -FRAC_PI_4 {
                            if let Some(_locked) = is_locked {
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
                            if let Some(_locked) = is_locked {
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
                            if let Some(_locked) = is_locked {
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
                            if let Some(_locked) = is_locked {
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

                    // Now that the MoveRequest knows what its final size can be, we push away other territories using this final size.
                    let mut other_territories = other_territories_query
                        .iter_many_mut(window_children);
                    while let Some(
                        (mut other_territory, _is_locked)
                    ) = other_territories.fetch_next() {

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
                },

                MoveRequestType::Drag | MoveRequestType::Resize => {
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
