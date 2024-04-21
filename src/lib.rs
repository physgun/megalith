pub mod input_manager;
pub mod components_ui;
pub mod systems_common;
pub mod systems_ui;
pub mod systems_egui;
pub mod resources_ui;
pub mod events_ui;

pub mod ui {
    use bevy::app::MainScheduleOrder;
    use bevy::ecs::schedule::ScheduleLabel;
    use bevy::prelude::*;
    use bevy::window::*;
    use leafwing_input_manager::prelude::*;

    use crate::input_manager::*;
    use crate::systems_common::*;
    use crate::components_ui::*;
    use crate::systems_egui::*;
    use crate::systems_ui::*;
    use crate::resources_ui::*;
    use crate::events_ui::*;
    

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct UpdateUIStateChanges;

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct UpdateUIInput;

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct UpdateUIWindowManagement;

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct UpdateUIPlaceholderManagement;

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct UpdateUITerritoryDrag;

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct UpdateUIDisplay;

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct UpdateUIDebug;

    // Plugin for the Territory Tabs UI, handling all initialization and updating.
    pub struct TerritoryTabsPlugin;
    impl Plugin for TerritoryTabsPlugin {
        fn build(&self, app: &mut App) {

            app
                // Stuff
                .insert_state(TerritoryTabsState::Natural)

                .add_plugins(InputManagerPlugin::<DevControls>::default())
                .init_resource::<ActionState<DevControls>>()
                .insert_resource(DevControls::default_input_map())

                .add_event::<TerritoryDragStarted>()
                .add_event::<TerritoryDragged>()
                .add_event::<TerritoryDragEnded>()

                .add_event::<TestChordJustPressed>()
                .add_event::<TestChordPressed>()
                .add_event::<TestChordJustReleased>()
                .add_event::<SpawnWindowKeyJustPressed>()

                // Startup
                .add_systems(Startup, initialize_ui_resources)
                .add_systems(Startup, initialize_egui_resources)

                // State Transitions
                .add_systems(OnEnter(TerritoryTabsState::MovingTabs),
                    setup_tab_move_placeholders)
                .add_systems(OnExit(TerritoryTabsState::MovingTabs), (
                    activate_placeholders
                        .before(cleanup_all_entities_with::<CleanupOnMovingTabExit>),
                    cleanup_all_entities_with::<CleanupOnMovingTabExit>
                ))

                // System Sets: Update
                .add_systems(Update, (
                    (
                        territory_tabs_state_drag_territories,
                        territory_tabs_state_tab_move
                    ).in_set(UpdateUIStateChanges),
                    (
                        test_spawn_window,
                        test_chord_pressed,
                        get_mouse_location
                    ).in_set(UpdateUIInput),
                    (
                        spawn_new_os_window
                            .before(configure_os_window),
                        configure_os_window
                            .run_if(on_event::<WindowCreated>())
                    ).in_set(UpdateUIWindowManagement),
                    (
                        check_placeholder_types_leaving_window
                            .run_if(on_event::<CursorLeft>())
                            .before(check_placeholder_types_entering_window),
                        check_placeholder_types_entering_window
                            .run_if(on_event::<CursorEntered>())
                            .before(check_placeholder_types_mouse_moving),
                        check_placeholder_types_mouse_moving
                            .run_if(on_event::<CursorMoved>())
                            .before(calculate_placeholder_data),
                        calculate_placeholder_data
                            .run_if(on_event::<CursorMoved>())
                    ).in_set(UpdateUIPlaceholderManagement),
                    (
                        determine_territory_drag_position
                            .run_if(on_event::<TerritoryDragged>())
                            .before(check_territory_drag_collision),
                        check_territory_drag_collision
                            .run_if(on_event::<TerritoryDragged>())
                            .before(check_window_drag_collision),
                        check_window_drag_collision
                            .run_if(on_event::<TerritoryDragged>()),
                    ).in_set(UpdateUITerritoryDrag),
                    (
                        egui_display_territories
                            .before(display_placeholders),
                        display_placeholders
                    ).in_set(UpdateUIDisplay),
                    (
                        display_debug_info
                    ).in_set(UpdateUIDebug)
                ))

                // Set Config: Update
                .configure_sets(Update, (
                    (
                        UpdateUIInput
                            .before(UpdateUIWindowManagement),
                        UpdateUIWindowManagement
                            .before(UpdateUIPlaceholderManagement)
                    ),
                    (
                        UpdateUIPlaceholderManagement
                            .run_if(not(in_state(TerritoryTabsState::Natural)))
                    ),
                    (
                        UpdateUIPlaceholderManagement
                            .before(UpdateUIDisplay)
                    ),
                    (
                        UpdateUITerritoryDrag
                            .run_if(in_state(TerritoryTabsState::DraggingTerritories))
                            .before(UpdateUIDisplay)
                    ),
                    (
                        UpdateUIDisplay
                            .before(UpdateUIDebug)
                    )
                ))

                ;
        }
    }
}