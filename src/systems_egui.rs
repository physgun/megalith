use std::ops::{Add, Sub};

use bevy::prelude::*;
use bevy_egui::egui::epaint::Shadow;
use bevy_egui::egui::{Align2, Rect as eguiRect, Style as eguiStyle, Vec2 as eguiVec2};
use bevy_egui::egui::{Pos2};
use bevy_egui::{egui, EguiContext};

use crate::components_ui::*;
use crate::events_ui::*;
use crate::input_manager::*;
use crate::resources_ui::*;
use crate::systems_common::TerritoryTabsState;
use crate::systems_ui::*;

// Insert egui related resources.
pub fn initialize_egui_resources (mut commands: Commands) {
    
}

// egui Debug Info Window until we get Tabs up and running.
pub fn display_debug_info(
    territory_tabs_current_state: Res<State<TerritoryTabsState>>,
    mut window_query: Query<(Entity, &Window, &mut EguiContext), With<EguiDisplay>>
) {
    for (window_entity, window, mut context) in &mut window_query {
        egui::Window::new("Debug Window")
            .title_bar(false)
            .anchor(Align2::LEFT_BOTTOM, eguiVec2::new(0.0, 0.0))
            .show(context.get_mut(), |ui| {
                let label = format!("Current State: {:?}", territory_tabs_current_state.get());
                ui.label(label)
            }
        );
    }
}

// How this shit work????
// Just gizmos for now!
pub fn display_placeholders(
    mut gizmos: Gizmos,
    placeholder_query: Query<&Placeholder>
) {
    for placeholder in & placeholder_query {
        match placeholder.placeholder_type {
            PlaceholderType::SpawnTerritory => {
                gizmos.rect_2d(
                    placeholder.worldspace_visual_rects[0].center(), 
                    0.0,
                    placeholder.worldspace_visual_rects[0].size(),
                    Color::RED
                );
                gizmos.rect_2d(
                    placeholder.worldspace_visual_rects[1].center(), 
                    0.0,
                    placeholder.worldspace_visual_rects[1].size(),
                    Color::WHITE
                );
            }
            _ => {}
        };
    }
}

// Toy function to get an idea of how this will all work.
// Try to see what egui can do as far as representing Territories goes.
pub fn egui_display_territories(
    mut gizmos: Gizmos,
    territory_settings: Res<TerritorySettings>,
    mut territory_drag_started: EventWriter<TerritoryDragStarted>,
    mut territory_dragged: EventWriter<TerritoryDragged>,
    mut territory_drag_ended: EventWriter<TerritoryDragEnded>,
    mut window_query: Query<(Entity, &Window, &mut EguiContext), With<EguiDisplay>>,
    territory_query: Query<(Entity, &Parent, &Territory)>
) {
    // Iterate through windows with EguiDisplay components, and their Territory children.
    for (window_entity, window, mut context) in &mut window_query {
        for (territory_entity, territory_parent, territory) in & territory_query {
            if territory_parent.get() == window_entity {

                // Both Bevy and egui have Vec2 / Rect types in their libraries! Need to convert from Bevy to egui.
                let egui_territory_rect = eguiRect::from_center_size(
                    Pos2::new( 
                        territory.screenspace_rect.center().x, 
                        territory.screenspace_rect.center().y
                    ),
                    eguiVec2::new( 
                        territory.screenspace_rect.size().x 
                        - 2.0 * territory_settings.inner_margins.x 
                        - territory_settings.spacing, 
                        territory.screenspace_rect.size().y 
                        - 2.0 * territory_settings.inner_margins.y 
                        - territory_settings.spacing,
                    )
                );

                gizmos.rect_2d(
                    territory.worldspace_rect.center(), 
                    0.0,
                    territory.worldspace_rect.size(),
                    Color::BLUE
                );
                let territory_frame = egui::Frame::window(&eguiStyle::default())
                    .shadow(Shadow::NONE)
                    .inner_margin(territory_settings.inner_margins.x)
                    ;

                egui::Window::new(territory_entity.index().to_string())
                    .title_bar(false)
                    .frame(territory_frame)
                    .pivot(Align2::CENTER_CENTER)
                    .default_rect(egui_territory_rect)
                    .current_pos(egui_territory_rect.center())
                    .show(context.get_mut(), |ui| {
                        ui.label("Blank Territory!");
                        let response = ui.allocate_response(
                            ui.available_size(), 
                            egui::Sense::click_and_drag()
                        );
                        if response.drag_started() {
                            let event_id = territory_drag_started.send(TerritoryDragStarted);
                        }
                        //let response = ui.interact(rect, id, egui::Sense::click_and_drag());
                        if response.dragged() {
                            let event_id = territory_dragged.send(TerritoryDragged { 
                                window_entity: window_entity,
                                dragged_entity: territory_entity,
                                mouse_delta: Vec2::new(response.drag_delta().x, -1.0 * response.drag_delta().y)
                            });
                        }
                        if response.drag_released() {
                            let event_id = territory_drag_ended.send(TerritoryDragEnded);
                        }
                    }
                );
            }
        }
    }
}
