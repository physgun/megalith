//! Contains all Events, Systems, SystemSets, and Plugins pertaining to a [`Territory`].

use std::f32::consts::FRAC_PI_4;
use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::window::*;
use bevy::render::camera::*;

use crate::components_territory::*;
use crate::display_territory::*;
use crate::display_territory_sickle::*;
use crate::input_manager::*;


pub struct TerritoryPlugin;

impl Plugin for TerritoryPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GlobalTerritorySettings>()
            .insert_state(TerritoryTabsMode::Operating)
            .add_event::<MoveRequestApplied>()
            .add_event::<TerritorySpawnRequest>()
            .add_event::<TerritoryDespawnRequest>()
            .add_systems(Startup, 
                configure_gizmos
            )
            .add_systems(Update, (
                (
                    configure_os_window
                        .run_if(on_event::<WindowCreated>()),
                )
                    .chain()
                    .in_set(WindowConfig),
                (
                    spawn_territory
                        .run_if(on_event::<TerritorySpawnRequest>()),
                    spawn_territory_sickle
                        .run_if(on_event::<TerritorySpawnRequest>()),
                    despawn_territory
                        .run_if(on_event::<TerritoryDespawnRequest>()),
                    display_debug_gizmos,
                )
                    .chain()
                    .in_set(TerritoryDisplay),
                (

                    (
                        empty_if_no_territories
                            .run_if(territory_removed.or_else(territory_spawned)),
                        test_delete_all_territories
                            .run_if(on_event::<RemoveTerritoriesKeyPressed>()),
                        update_territory_base_node,
                        territory_drag_move_request_sickle,
                        territory_resize_move_request_sickle
                    ) 
                        .chain()
                        .in_set(TerritoryUpdateState),
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
    /// The [`Window`] that the new [`Territory`] will be a child of.
    pub window_entity: Entity,
    /// Where the [`Territory`] should be.
    pub expanse: RectKit,
    /// How the [`Territory`] should be represented in UI.
    pub display_library: DisplayLibrary
}

/// Sent when a system has commanded a [`Territory`] to despawn.
#[derive(Event)]
pub struct TerritoryDespawnRequest {
    /// [`Entity`] to be despawned.
    pub despawned_territory: Entity
}

/// Make debug gizmos not be covered up by nodes.
pub fn configure_gizmos (
    mut gizmo_central_resource: ResMut<GizmoConfigStore>
) {
    let (config, _) = gizmo_central_resource.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -1.0;
}

/// Debug gizmos!
pub fn display_debug_gizmos (
    mut gizmos: Gizmos,
    territory_query: Query<&Territory>
) {
    for territory in & territory_query {
        gizmos.rect_2d(
            territory.expanse.worldspace().center(), 
            0.0,
            territory.expanse.worldspace().size(),
            Color::BLUE,
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

            commands.spawn((
                Name::new("[ROOT NODE] Territory Tabs Window Root Node"),
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgb_u8(21, 52, 72)),
                    ..default()
                },
                TargetCamera(child_camera),
                TerritoryTabsUIRoot {
                    associated_window_entity: event.window
                }
            ));
    
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
                //warn!("Unexpected transition: Empty -> Empty"); 
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
    mut remove_territories_key_pressed: EventReader<RemoveTerritoriesKeyPressed>,
    mut despawn_territory_request:EventWriter<TerritoryDespawnRequest>,
    window_query: Query<&Children, With<Window>>,
    territory_query: Query<Entity, With<Territory>>
) {
    for _event in remove_territories_key_pressed.read() {
        for window_children in & window_query {
            let mut territories = territory_query.iter_many(window_children);
            while let Some(territory_entity) =  territories.fetch_next(){
                despawn_territory_request.send(TerritoryDespawnRequest { despawned_territory: territory_entity });
            }
        }
    }
}


















/// Initial examination of all [`DragRequest`]s attached to [`Territory`] entities.  
///   
/// Other than the basic checks, the big operation here is to determine what [Territory]s
/// are connected to this one directly or indirectly down the graph and add a [`DragTerritoryGroup`]
/// marker component to them for ease of later processing. This is so collision logic
/// can be run on all connected [`Territory`]s and they all appear to move as one connected whole.
pub fn territory_drag_request_eval (
    mut commands: Commands,
    dragging_territory_query: Query<(Entity, &Territory, Option<&Locked>, &DragRequest)>,
    potential_neighbor_query: Query<&CardinalConnections, With<Territory>>
) {
    let Ok(
        (territory_entity, territory, territory_locked, drag_request)
        ) = dragging_territory_query.get_single() else {
        error!("Drag request systems activated but drag query did not have single entity!");
        return;
    };

    // Locked Territories don't move anywhere.
    if territory_locked.is_some() {
        debug!("Removed a DragRequest from a locked Territory!");
        commands.entity(territory_entity).remove::<DragRequest>();
        return;
    }

    // Catch any zero-movement requests.
    if drag_request.proposed_expanse().worldspace() == territory.expanse().worldspace() {
        debug!("Removed a zero-movement DragRequest from a Territory!");
        commands.entity(territory_entity).remove::<DragRequest>();
        return;
    }
    
    // Depth first traversal to collect all territory entities connected to the one with the DragRequest.
    let mut to_be_traversed_entities: Vec<Entity> = Vec::new();
    let mut collected_entities: Vec<Entity> = Vec::new();

    // Add the OG DragRequest Territory to the stack.
    to_be_traversed_entities.push(territory_entity);
    debug!("[DFS] Added DragRequest Territory to stack.");

    // Find all connections and add them to the dragged territory group.
    while let Some(current_entity) =  to_be_traversed_entities.pop() {
        collected_entities.push(current_entity);
        debug!("[DFS] Popped Territory off of the stack and added to visited.");

        commands.entity(current_entity).insert(DragTerritoryGroup);

        let Ok(current_connections) = potential_neighbor_query.get(current_entity) else {
            error!("[DFS] CardinalConnections component get error!");
            continue;
        };

        for next_entity in current_connections.get_all_vec() {
            if collected_entities.contains(&next_entity) { 
                debug!("[DFS] Popped Territory neighbor already visited.");
                continue; 
            }
            to_be_traversed_entities.push(next_entity);
            debug!("[DFS] Popped Territory neighbor pushed to stack.");
        }
    }
}

/// Initial examination of all [`ResizeRequest`]s attached to [`Territory`] entities.  
///   
/// Basic sanity checks and a depth first traversal to find connected [`Territory`]s
/// with similar and opposite resizing, to be marked with [`AdvancingTerritoryGroup`] and [`RetreatingTerritoryGroup`].
pub fn territory_resize_request_eval (
    mut commands: Commands,
    resizing_territory_query: Query<(Entity, &Territory, &CardinalConnections, Option<&Locked>, &ResizeRequest)>,
    potential_neighbor_query: Query<(&CardinalConnections, &Territory, Option<&Locked>), Without<ResizeRequest>>
) {
    let Ok(
        (territory_entity, territory, initial_connections, territory_locked, resize_request)
        ) = resizing_territory_query.get_single() else {
        error!("Resize request systems activated but resize query did not have single entity!");
        return;
    };

    // Locked Territories don't change size.
    if territory_locked.is_some() {
        debug!("Removed a ResizeRequest from a locked Territory!");
        commands.entity(territory_entity).remove::<ResizeRequest>();
        return;
    }

    // Catch any zero-movement requests. These are common on ResizeRequests when the user drags parallel to the resize bar.
    if resize_request.proposed_expanse().worldspace() == territory.expanse().worldspace() {
        commands.entity(territory_entity).remove::<ResizeRequest>();
        return;
    }

    // If our OG DragRequesting Territory is a corner or other multi-side resize with a retreating side,
    // there is a possibility of collisions between the OG's connecting Territories.
    // More efficient to handle this special case here and now rather than later.
    // Thankfully, only the OG territory will do any multi-side resizing. Any downstream effects are all one-sided.
    if resize_request.resize_direction().is_multi_side_resize() && resize_request.resize_direction().has_any_retreating() {

        // Collection of screenspace neighbor rects modified by the impending resize, to be checked for collisions.
        let mut neighbor_rects: Vec<Rect> = Vec::new();

        // For each basic direction our special multi-side resize affects:
        for cardinal_direction in resize_request.resize_direction().get_cardinal_directions() {

            // Get all entities connected to that specific basic direction.
            let neighbor_entities = initial_connections.get_resize_direction_vec(cardinal_direction);

            // For each of these entity's territories:
            for (_, checked_territory, _) in potential_neighbor_query.iter_many(neighbor_entities) {

                // Push the modifed rect, noting that the connecting rect will have opposite border movement.
                neighbor_rects.push(cardinal_direction.get_opposite().apply_to_rect(checked_territory.expanse().screenspace()));

            }
        }

        // Check unique pairs of the modifed rects for collisions.
        // There are many options for what to do if a collision occurs.
        // The least annoying option for the user is to cancel the ResizeRequest.
        for (index, rect1) in neighbor_rects.iter().enumerate() {
            for rect2 in &neighbor_rects[index + 1..] {
                if rect1.intersect(*rect2).is_empty() { 
                    continue; 
                }
                else { 
                    commands.entity(territory_entity).remove::<ResizeRequest>(); 
                    return;
                }
            }
        }

    }

    // For easier interaction with Locked territories, 
    // it's best to have an individual DFS per cardinal direction for multi-side resizing.
    for cardinal_direction in resize_request.resize_direction().get_cardinal_directions() {

        // Depth first traversal like drag, but we only care about connections that share an opposing advancing or retreating border.
        let mut to_be_traversed_entities: Vec<(ResizeDirection, Entity)> = Vec::new();
        let mut collected_entities: Vec<(ResizeDirection, Entity)> = Vec::new();

        // Push OG territory's cardinal side to stack
        to_be_traversed_entities.push((cardinal_direction, territory_entity));
        debug!("[DFS] Added OG ResizeRequest Territory side {:?} to stack.", cardinal_direction);

        // Find the connections who will be affected by the ResizeRequest.
        // Mark them as part of an advancing or retreating group of territories.
        while let Some((resize_direction, current_entity)) =  to_be_traversed_entities.pop() {
            // We've visited this territory's side, so add to list of ones we've already seen.
            collected_entities.push((resize_direction, current_entity));
            debug!("[DFS] Popped Territory with side {:?} off stack and added to visited.", resize_direction);

            // Get the connections of the just-popped territory, and see if its locked too. 
            let Ok((current_connections, _, locked
            )) = potential_neighbor_query.get(current_entity) else {
                // Failure here would mean a more broad-scoped component error.
                error!("[DFS] CardinalConnections component get error!");
                continue;
            };

            // A locked territory means this entire side's resize chain is invalid. 
            // But, any other cardinal directions could still be valid, so we can't remove the ResizeRequest entirely.
            // Instead, remove all group components from the collection of visited entities and bail.
            if locked.is_some() {
                for (visited_direction, visited_entity) in collected_entities {
                    match visited_direction.get_single_magnitude() {
                        ResizeMagnitude::None => { 
                            warn!("{:?} somehow in collection of DFS visited entities??", ResizeMagnitude::None);
                        }
                        ResizeMagnitude::Advancing(_) => {
                            commands.entity(visited_entity).remove::<AdvancingTerritoryGroup>();
                        }
                        ResizeMagnitude::Retreating(_) => {
                            commands.entity(visited_entity).remove::<RetreatingTerritoryGroup>();
                        }
                    }
                }
                break;
            }

            // Add to group depending on resize magnitude.
            match resize_direction.get_single_magnitude() {
                ResizeMagnitude::None => { warn!("Popped resize territory had {:?}!", ResizeMagnitude::None) }
                ResizeMagnitude::Advancing(_) => { 
                    commands.entity(current_entity).insert(AdvancingTerritoryGroup(resize_direction)); 
                }
                ResizeMagnitude::Retreating(_) => {
                    commands.entity(current_entity).insert(RetreatingTerritoryGroup(resize_direction));
                }
            }

            // Add relevant connections to the stack to be popped later. We'll need the opposite ResizeDirection:
            let opposite_direction = resize_direction.get_opposite();
            for next_entity in current_connections.get_resize_direction_vec(resize_direction) {
                if collected_entities.contains(&(opposite_direction, next_entity)) { 
                    debug!("[DFS] Popped Territory neighbor already visited.");
                    continue; 
                }

                // Push unvisited, relevant connection to stack.
                to_be_traversed_entities.push((opposite_direction, next_entity));
                debug!("[DFS] Popped Territory neighbor with side {:?} pushed to stack.", opposite_direction);
            }
        } 
    }

}

/// Handle [`DragRequest`]s that try to move the [`Territory`] beyond the window edge. Linked Territories will also need to be checked.
///   
/// This is better handled in **screenspace**.
pub fn territory_drag_request_window_edge (
    window_query: Query<(&Window, &Children), With<TerritoryTabs>>,
    dragging_territories_query: Query<(&DragRequest, &Territory)>,
    connected_territories_query: Query<(&Territory, &CardinalConnections), Without<DragRequest>>
) {
    for (window, window_children) in & window_query {

        let mut dragging_territories = dragging_territories_query.iter_many(window_children);

        while let Some((drag_request, territory)) = dragging_territories.fetch_next() {

            let (window_width, window_height) = (window.width(), window.height());



            // Is the proposed RectKit in the window? 
            if drag_request.proposed_expanse().is_inside_screenspace_window(window_width, window_height) {
                continue;
            }

            // Left
            if drag_request.proposed_expanse().screenspace().min.x < 0.0 {
                let delta_x = -1.0 * drag_request.proposed_expanse().screenspace().min.x;
                drag_request.proposed_expanse().move_screenspace_pos(delta_x, 0.0, window_width, window_height);
            } // Top
            if drag_request.proposed_expanse().screenspace().min.y < 0.0 {
                let delta_y = -1.0 * drag_request.proposed_expanse().screenspace().min.y;
                drag_request.proposed_expanse().move_screenspace_pos(0.0, delta_y, window_width, window_height);
            } // Right
            if drag_request.proposed_expanse().screenspace().max.x > window_width {
                let delta_x = window_width - drag_request.proposed_expanse().screenspace().max.x;
                drag_request.proposed_expanse().move_screenspace_pos(delta_x, 0.0, window_width, window_height);
            } // Bottom
            if drag_request.proposed_expanse().screenspace().max.y > window_height {
                let delta_y = window_height - drag_request.proposed_expanse().screenspace().max.y;
                drag_request.proposed_expanse().move_screenspace_pos(0.0, delta_y, window_width, window_height);
            }
        }
    }
}

/// Handle [`ResizeRequest`]s that try to expand the [`Territory`] beyond the window edge.  
///   
/// This is better handled in **screenspace**.
pub fn territory_resize_request_window_edge (
    window_query: Query<(&Window, &Children), With<TerritoryTabs>>,
    mut resizing_territories_query: Query<&ResizeRequest, With<Territory>>
) {
    for (window, window_children) in & window_query {

        let mut resizing_territories = resizing_territories_query.iter_many_mut(window_children);

        while let Some(resize_request) = resizing_territories.fetch_next() {

            let (window_width, window_height) = (window.width(), window.height());

            if resize_request.proposed_expanse().is_inside_screenspace_window(window_width, window_height) {
                continue;
            }

            let window_rect = Rect::from_corners(Vec2::ZERO, Vec2::new(window_width, window_height));
            let new_rect = window_rect.intersect(resize_request.proposed_expanse().screenspace());
            resize_request.proposed_expanse().set_screenspace(new_rect, window_width, window_height);
        }
    }
}
























/// Initial check of all [`Territory`]s who have a [`MoveRequest`] component and catch any odd requests.
/// Any [`Locked`] [`Territory`]s will have their [`MoveRequest`] component removed.
pub fn territory_move_eval_type (
    mut commands: Commands,
    window_query: Query<&Children, (With<Window>, With<TerritoryTabs>)>,
    mut moving_territories_query: Query<(Entity, &Territory, Option<&Locked>, &mut MoveRequest)>
) {
    for window_children in & window_query {
        let mut moving_territories = moving_territories_query.iter_many_mut(window_children);
        while let Some(
            (territory_entity, territory, territory_locked, move_request)
        ) = moving_territories.fetch_next() {

            // A Locked Territory won't process any MoveRequest.
            if let Some(_locked) = territory_locked {
                commands.entity(territory_entity).remove::<MoveRequest>();
                continue;
            }

            if move_request.proposed_expanse.worldspace() == territory.expanse.worldspace() {
                commands.entity(territory_entity).remove::<MoveRequest>();
                debug!("MoveRequest found with identical rect to existing rect, and was removed!");
                continue;
            }

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
                    if window_rect.contains(move_request.proposed_expanse.worldspace().min)
                    && window_rect.contains(move_request.proposed_expanse.worldspace().max) {continue;}
    
                    if move_request.proposed_expanse.worldspace().min.x < window_rect.min.x {
                        let delta_x = window_rect.min.x - move_request.proposed_expanse.worldspace().min.x;
                        move_request.proposed_expanse.move_worldspace_pos(
                            delta_x,
                            0.0,
                            window.width(),
                            window.height()
                        );
                    }
                    if move_request.proposed_expanse.worldspace().min.y < window_rect.min.y {
                        let delta_y = window_rect.min.y - move_request.proposed_expanse.worldspace().min.y;
                        move_request.proposed_expanse.move_worldspace_pos(
                            0.0,
                            delta_y,
                            window.width(),
                            window.height()
                        );
                    }
                    if move_request.proposed_expanse.worldspace().max.x > window_rect.max.x {
                        let delta_x = window_rect.max.x - move_request.proposed_expanse.worldspace().max.x;
                        move_request.proposed_expanse.move_worldspace_pos(
                            delta_x,
                            0.0,
                            window.width(),
                            window.height()
                        );
                    }
                    if move_request.proposed_expanse.worldspace().max.y > window_rect.max.y {
                        let delta_y = window_rect.max.y - move_request.proposed_expanse.worldspace().max.y;
                        move_request.proposed_expanse.move_worldspace_pos(
                            0.0,
                            delta_y,
                            window.width(),
                            window.height()
                        );
                    }
                },
                MoveRequestType::Resize(_) => {
                    let inbounds_rect = window_rect.intersect(move_request.proposed_expanse.worldspace());

                    move_request.proposed_expanse.set_worldspace(
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
    territory_settings: Res<GlobalTerritorySettings>,
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

                        let conflict_rect = move_request.proposed_expanse.worldspace()
                            .intersect(other_territory.expanse.worldspace());
                        if conflict_rect.is_empty() {continue;}

                        // If the user goes nuts, they can drag Territories fast enough that the conflict rect
                        // is entirely contained inside our Territory rect. Remaining space handles that case. Mostly.
                        // TODO: Handle that case better than mostly.
                        if conflict_rect.height() >= conflict_rect.width() {

                            if move_request.proposed_expanse.worldspace().center().x 
                            >= other_territory.expanse.worldspace().center().x {
                                let remaining_space = other_territory.expanse.worldspace().max.x - conflict_rect.max.x;
                                move_request.proposed_expanse.move_worldspace_pos(
                                    conflict_rect.width() + remaining_space,
                                    0.0,
                                    window.width(),
                                    window.height()
                                );
                            }
                            else {
                                let remaining_space = conflict_rect.min.x - other_territory.expanse.worldspace().min.x;
                                move_request.proposed_expanse.move_worldspace_pos(
                                    -1.0 * conflict_rect.width() - remaining_space,
                                    0.0,
                                    window.width(),
                                    window.height()
                                );
                            }
                        }
                        else {

                            if move_request.proposed_expanse.worldspace().center().y 
                            >= other_territory.expanse.worldspace().center().y {
                                let remaining_space = other_territory.expanse.worldspace().max.y - conflict_rect.max.y;
                                move_request.proposed_expanse.move_worldspace_pos(
                                    0.0,
                                    conflict_rect.height() + remaining_space,
                                    window.width(),
                                    window.height()
                                );
                            }
                            else {
                                let remaining_space = conflict_rect.min.y - other_territory.expanse.worldspace().min.y;
                                move_request.proposed_expanse.move_worldspace_pos(
                                    0.0,
                                    -1.0 * conflict_rect.height() - remaining_space,
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

                        let conflict_rect = move_request.proposed_expanse.worldspace()
                            .intersect(other_territory.expanse.worldspace());
                        if !conflict_rect.is_empty() {
                            warn!("Drag-type MoveRequest still found conflicts after processing. MoveRequest removed!");
                            commands.entity(territory_entity).remove::<MoveRequest>();
                        }
                    }
                },

                MoveRequestType::Resize(_) => {
                    let mut other_territories = other_territories_query
                        .iter_many_mut(window_children);
                    while let Some(
                        (other_territory, is_locked)
                    ) = other_territories.fetch_next() {
                            
                        let conflict_rect = move_request.proposed_expanse.worldspace()
                            .intersect(other_territory.expanse.worldspace());
                        if conflict_rect.is_empty() {continue;}

                        // Find the conflict_rect's sector, which determines what direction we pared back proposed resize.
                        let conflict_angle = (
                            move_request.proposed_expanse.worldspace().center().y - conflict_rect.center().y)
                            .atan2(
                            move_request.proposed_expanse.worldspace().center().x - conflict_rect.center().x);

                        // Cycle through and see, first, how far we can move our resize, paring back as necessary.
                        // Don't move away other Territories yet. Some might be locked!

                        // Right
                        if conflict_angle <= FRAC_PI_4 && conflict_angle >= -FRAC_PI_4 {
                            if let Some(_locked) = is_locked {
                                move_request.proposed_expanse.move_worldspace_corners(
                                    Vec2::ZERO, 
                                    Vec2::new(-1.0 * conflict_rect.width(), 0.0), 
                                    window.width(), 
                                    window.height()
                                );
                                continue;
                            }

                            let conflict_overreach = conflict_rect.width()
                                - (other_territory.expanse.worldspace().width() - territory_settings.min_size.x);

                            if conflict_overreach > 0.0 {
                                move_request.proposed_expanse.move_worldspace_corners(
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
                                move_request.proposed_expanse.move_worldspace_corners(
                                    Vec2::ZERO, 
                                    Vec2::new(0.0, -1.0 * conflict_rect.height()), 
                                    window.width(), 
                                    window.height()
                                );
                                continue;
                            }

                            let conflict_overreach = conflict_rect.height()
                                - (other_territory.expanse.worldspace().height() - territory_settings.min_size.y);

                            if conflict_overreach > 0.0 {
                                move_request.proposed_expanse.move_worldspace_corners(
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
                                move_request.proposed_expanse.move_worldspace_corners(
                                    Vec2::new(1.0 * conflict_rect.width(), 0.0), 
                                    Vec2::ZERO, 
                                    window.width(), 
                                    window.height()
                                );
                                continue;
                            }

                            let conflict_overreach = conflict_rect.width()
                                - (other_territory.expanse.worldspace().width() - territory_settings.min_size.x);

                            if conflict_overreach > 0.0 {
                                move_request.proposed_expanse.move_worldspace_corners(
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
                                move_request.proposed_expanse.move_worldspace_corners(
                                    Vec2::new(0.0, 1.0 * conflict_rect.height()), 
                                    Vec2::ZERO, 
                                    window.width(), 
                                    window.height()
                                );
                                continue;
                            }

                            let conflict_overreach = conflict_rect.height()
                                - (other_territory.expanse.worldspace().height() - territory_settings.min_size.y);

                            if conflict_overreach > 0.0 {
                                move_request.proposed_expanse.move_worldspace_corners(
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

                        let conflict_rect = move_request.proposed_expanse.worldspace()
                            .intersect(other_territory.expanse.worldspace());
                        if conflict_rect.is_empty() {continue;}

                        // Find the conflict_rect's sector, which determines what direction we resize the other Territory.
                        let conflict_angle = (
                            other_territory.expanse.worldspace().center().y - conflict_rect.center().y)
                            .atan2(
                            other_territory.expanse.worldspace().center().x - conflict_rect.center().x);

                        // Second run-through to push other Territories out of our, now valid, resize MoveRequest.
                        // Don't forget to invert the direction of resize, 
                        // since the proposed resize's right is the other Territory's left.

                        // Right
                        if conflict_angle <= FRAC_PI_4 && conflict_angle >= -FRAC_PI_4 {
                            other_territory.expanse.move_worldspace_corners(
                                Vec2::new(1.0 * conflict_rect.width(), 0.0),
                                Vec2::ZERO,
                                window.width(),
                                window.height()
                            );
                        } 
                        // Top
                        else if conflict_angle >= FRAC_PI_4 && conflict_angle <= 3.0 * FRAC_PI_4 {
                            other_territory.expanse.move_worldspace_corners(
                                Vec2::new(0.0, 1.0 * conflict_rect.height()),
                                Vec2::ZERO,
                                window.width(),
                                window.height()
                            );
                        }
                        // Left (atan2 is discontinuous at PI, as its range is -PI to PI)
                        else if (conflict_angle >= 3.0 * FRAC_PI_4 && conflict_angle <= PI)
                            || (conflict_angle >= -PI && conflict_angle <= -3.0 * FRAC_PI_4) {
                            other_territory.expanse.move_worldspace_corners(
                                Vec2::ZERO,
                                Vec2::new(-1.0 * conflict_rect.height(), 0.0),
                                window.width(),
                                window.height()
                            );
                        }
                        // Down
                        else if conflict_angle >= -3.0 * FRAC_PI_4 && conflict_angle <= -FRAC_PI_4 {
                            other_territory.expanse.move_worldspace_corners(
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

                MoveRequestType::Drag | MoveRequestType::Resize(_) => {
                    territory.expanse.set_worldspace(
                        move_request.proposed_expanse.worldspace(), 
                        window.width(), 
                        window.height()
                    );
                    commands.entity(territory_entity).remove::<MoveRequest>();
                }
            }
        }
    }
}

