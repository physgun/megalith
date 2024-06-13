pub mod input_manager;
pub mod components_common;
pub mod components_ui;
pub mod systems_common;
pub mod systems_ui;
pub mod systems_egui;
pub mod resources_ui;

pub mod components_territory;
pub mod systems_territory;
pub mod display_territory;
pub mod display_territory_sickle;

pub mod ui {
    use bevy::prelude::*;
    use leafwing_input_manager::prelude::*;

    use crate::input_manager::*;
    use crate::systems_common::*;
    use crate::systems_egui::*;
    use crate::systems_ui::*;
    
    use crate::systems_territory::*;
    

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct UpdateUIStateChanges;

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct UpdateUIInput;

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct UpdateUIWindowManagement;

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct UpdateUIPlaceholderManagement;

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct UpdateUITerritoryMove;

    #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
    pub struct UpdateUIStateBehavior;

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
                .add_plugins(TerritoryPlugin)
                .insert_state(TerritoryTabsState::Natural)

                .add_plugins(InputManagerPlugin::<DevControls>::default())
                .init_resource::<ActionState<DevControls>>()
                .insert_resource(DevControls::default_input_map())

                .add_event::<TestChordJustPressed>()
                .add_event::<TestChordPressed>()
                .add_event::<TestChordJustReleased>()
                .add_event::<SpawnWindowKeyJustPressed>()
                .add_event::<RemoveTerritoriesKeyPressed>()

                // Test system
                .add_systems(Update, 
                    test_delete_all_territories_just_pressed
                )

                // Startup
                .add_systems(Startup, initialize_ui_resources)

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
                        test_spawn_window,
                        test_chord_pressed,
                        get_mouse_location
                    ).in_set(UpdateUIInput),
                    // (
                    //    
                    // ).in_set(UpdateUIDisplay),
                    (
                        spawn_new_os_window
                    ).in_set(UpdateUIWindowManagement),
                    (
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
                    ).in_set(UpdateUIStateBehavior),
                    (
                        display_debug_info_with_egui,
                        display_placeholders_egui
                    ).in_set(UpdateUIDebug),
                    (
                        territory_tabs_main_state_exit
                            .before(territory_tabs_main_state_enter),
                        territory_tabs_main_state_enter
                    ).in_set(UpdateUIStateChanges)
                ))

                // Set Config: Update
                .configure_sets(Update, (
                    (
                        UpdateUIInput,
                    ).before(UpdateUIDisplay),
                    (
                        UpdateUIDisplay
                    ).before(UpdateUIWindowManagement),
                    (
                        UpdateUIWindowManagement,
                    ).before(UpdateUIStateBehavior),
                    (
                        UpdateUIStateBehavior
                    ).before(UpdateUIDebug),
                    (
                        UpdateUIDebug
                    ).before(UpdateUIStateChanges),
                    (
                        UpdateUIStateChanges
                    )
                ));
        }
    }
}