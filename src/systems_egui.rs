use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::components_ui::*;
use crate::resources_ui::*;
use crate::systems_common::TerritoryTabsState;

// Insert egui related resources.
pub fn initialize_egui_resources (mut commands: Commands) {
    
}

// egui Debug Info Window until we get Tabs up and running.
pub fn display_debug_info_with_egui(
    territory_tabs_current_state: Res<State<TerritoryTabsState>>,
    mut window_query: Query<(Entity, &Window, &mut EguiContext)>
) {
    for (window_entity, window, mut context) in &mut window_query {
        egui::Window::new("Debug Window")
            .title_bar(false)
            .default_pos(egui::Pos2::new(0.0, 0.0))
            .default_size(egui::Vec2::new(200.0, 25.0))
            .show(context.get_mut(), |ui| {
                let main_state_label = format!("Current State: {:?}", territory_tabs_current_state.get());
                ui.label(main_state_label);

                ui.allocate_space(ui.available_size());
            }
        );
    }
}

// How this shit work????
// Just gizmos for now!
pub fn display_placeholders_egui(
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

pub fn display_territory_egui (
    mut commands: Commands,
    territory_settings: Res<TerritorySettings>,
    mut window_query: Query<(Entity, &Window, &mut EguiContext)>,
    territory_query: Query<(Entity, &Parent, &Territory, &DisplayLibrary), Without<Overlay>>,
    overlay_query: Query<(Entity, &Parent, &Territory, &DisplayLibrary), With<Overlay>>
) {
    for (
        window_entity, 
        window, 
        mut egui_context
    ) in &mut window_query {
        for (
            territory_entity, 
            territory_parent, 
            territory, 
            territory_display
        ) in & territory_query {
            // Iterate through all Territory components with DisplayLibrary::BevyEgui and add 
            // egui ui to their Parent window's context.
            if territory_parent.get() == window_entity && matches!(territory_display, DisplayLibrary::BevyEgui) {
                // egui doesn't really like to paint a window to your exact specifications.
                // Some fighting and hair-pulling may be required. 
                let requested_egui_rect = egui::Rect::from_center_size(
                    egui::Pos2::new(
                        territory.screenspace_rect().center().x, 
                        territory.screenspace_rect().center().y
                    ), 
                    egui::Vec2::new(
                        territory.screenspace_rect().size().x
                        - territory_settings.inner_margins.x * 2.0
                        - territory_settings.spacing, 
                        territory.screenspace_rect().size().y
                        - territory_settings.inner_margins.y * 2.0
                        - territory_settings.spacing
                    )
                );

                // Bunch of settings.
                let main_window_title = territory_entity.index().to_string();
                let territory_style = egui::Style::default();
                let debug_fill = egui::Color32::from_rgba_premultiplied(50, 50, 50, 25);
                let territory_frame_stroke = 1.15;
                let territory_frame = egui::Frame::window(&territory_style)
                    .shadow(egui::epaint::Shadow::NONE)
                    .stroke((territory_frame_stroke, egui::Color32::from_gray(60)))
                    .fill(debug_fill)
                    .inner_margin(territory_settings.inner_margins.x);

                // egui resize is buggy and won't play nicely with user desires. 
                // This hacky solution, a scroll area inside a resize inside a window, is weird but actually works okay.
                let tab_contents_resize_area = egui::Resize::default()
                    .id_source(format!("{} Tab Contents Resize Area", &main_window_title))
                    .default_size(requested_egui_rect.size())
                    .min_width(territory_settings.min_size.x
                    - 2.0 * territory_settings.inner_margins.x
                    - 1.0 * territory_settings.spacing)
                    .min_height(territory_settings.min_size.y
                    - 2.0 * territory_settings.inner_margins.y
                    - 1.0 * territory_settings.spacing)
                    .max_width(window.width()
                    - 2.0 * territory_settings.inner_margins.x
                    - 1.0 * territory_settings.spacing)
                    .max_height(window.height()
                    - 2.0 * territory_settings.inner_margins.x
                    - 1.0 * territory_settings.spacing);

                egui::Window::new(&main_window_title)
                    .title_bar(false)
                    .frame(territory_frame)
                    .pivot(egui::Align2::CENTER_CENTER)
                    .min_width(territory_settings.min_size.x)
                    .min_height(territory_settings.min_size.y) // Doesn't appear to do anything??
                    .max_width(window.width()) // Doesn't appear to do anything??
                    .max_height(window.height()) // Doesn't appear to do anything??
                    .default_size(requested_egui_rect.size())
                    .current_pos(requested_egui_rect.center())
                    .resizable(false)
                    .show(egui_context.get_mut(), |ui| {

                        tab_contents_resize_area.show(ui, |ui| {

                            egui::ScrollArea::both()
                                .id_source(format!("{} Encapsulating Scroll Area", &main_window_title))
                                .min_scrolled_height(1.0)
                                .min_scrolled_width(1.0)
                                .drag_to_scroll(false)
                                .show(ui, |ui| {

                                    ui.allocate_space(ui.available_size());
                                    let bg_response = ui.interact_bg(egui::Sense::click_and_drag());

                                    // "actual egui rect" results may vary DRAMATICALLY and for DIFFICULT TO DISCERN REASONS.
                                    let actual_egui_rect = egui::Rect::from_center_size(
                                        ui.clip_rect().center(), 
                                        egui::Vec2::new(
                                            ui.clip_rect().size().x - 6.0, // Why -6.0? Who knows??
                                            ui.clip_rect().size().y - 6.0  
                                        )
                                    );

                                    let mut delta_size = Vec2::new(
                                        actual_egui_rect.width() - requested_egui_rect.width(), 
                                        actual_egui_rect.height() - requested_egui_rect.height()
                                    );

                                    delta_size.x = f32::trunc(delta_size.x * 100.0) / 100.0;
                                    delta_size.y = f32::trunc(delta_size.y * 100.0) / 100.0;

                                    // If a drag or a change in size was detected, attach a MoveRequest.
                                    // Will conveniently overwrite an old MoveRequest should one exist, which it shouldn't!
                                    if bg_response.dragged() || delta_size.abs().length() > 0.0 {
                                        commands.entity(territory_entity).insert(
                                            MoveRequest::from_screenspace_rect(
                                                Rect::from_corners(
                                                    Vec2::new(
                                                        actual_egui_rect.min.x, 
                                                        actual_egui_rect.min.y
                                                    ), 
                                                    Vec2::new(
                                                        actual_egui_rect.max.x, 
                                                        actual_egui_rect.max.y
                                                    )
                                                )
                                            ).screen_to_world(window.width(), window.height()).clone()
                                        );
                                    }

                                    
                                })
                        })
                    });
            };
        }
    }
}