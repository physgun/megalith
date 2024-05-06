use bevy::prelude::*;
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
pub fn display_debug_info_with_egui(
    territory_tabs_current_state: Res<State<TerritoryTabsState>>,
    mut window_query: Query<(Entity, &Window, &mut EguiContext), With<EguiDisplay>>
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
pub fn display_placeholders_with_egui(
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
pub fn display_territories_with_egui(
    territory_settings: Res<TerritorySettings>,
    tab_settings: Res<TabSettings>,
    territory_tabs_current_state: Res<State<TerritoryTabsState>>,
    mut territory_drag_started: EventWriter<TerritoryDragStarted>,
    mut territory_dragged: EventWriter<TerritoryDragged>,
    mut territory_drag_ended: EventWriter<TerritoryDragEnded>,
    mut territory_resize_started: EventWriter<TerritoryResizeStarted>,
    mut territory_resized: EventWriter<TerritoryResizing>,
    mut territory_resize_ended: EventWriter<TerritoryResizeEnded>,
    mut window_query: Query<(Entity, &Window, &mut EguiContext), With<EguiDisplay>>,
    territory_query: Query<(Entity, &Parent, &Territory)>
) {
    // Iterate through windows with EguiDisplay components, and their Territory children.
    for (window_entity, window, mut context) in &mut window_query {
        for (territory_entity, territory_parent, territory) in & territory_query {
            if territory_parent.get() == window_entity {

                let egui_territory_rect = egui::Rect::from_center_size(
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
                ).shrink(0.0);

                let territory_min_size = egui::Vec2::new(
                    territory_settings.min_size.x, 
                    territory_settings.min_size.y
                );

                let ctx = context.get_mut();
                ctx.style_mut(|style| {
                    style.wrap = Some(false);
                });
                let main_window_title = territory_entity.index().to_string();

                let territory_style = egui::Style::default();

                let debug_fill = egui::Color32::from_rgba_premultiplied(50, 50, 50, 25);

                let territory_frame_stroke = 1.15;

                let territory_frame = egui::Frame::window(&territory_style)
                    .shadow(egui::epaint::Shadow::NONE)
                    .stroke((territory_frame_stroke, egui::Color32::from_gray(60)))
                    .fill(debug_fill)
                    .inner_margin(territory_settings.inner_margins.x);

                let tab_button = egui::Button::new("Tab")
                    .wrap(false)
                    .min_size(egui::Vec2::splat(1.0));

                let tab_bar_scroll_area = egui::ScrollArea::horizontal()
                    .id_source(format!("{} Tab Bar Scroll Area", &main_window_title))
                    .max_width(egui_territory_rect.width() - territory_settings.inner_margins.x)
                    .max_height(tab_settings.min_size.y)
                    .min_scrolled_width(1.0)
                    .min_scrolled_height(1.0)
                    .drag_to_scroll(false);

                let tab_border = egui::Separator::default().shrink(5.0).spacing(2.0);

                let tab_contents_scroll_area = egui::ScrollArea::both()
                    .id_source(format!("{} Tab Contents Scroll Area", &main_window_title))
                    .max_width(egui_territory_rect.width() - territory_settings.inner_margins.x)
                    .drag_to_scroll(false)
                    .scroll2(true)
                    .min_scrolled_height(1.0)
                    .min_scrolled_width(1.0);

                let tab_contents_resize_area = egui::Resize::default()
                    .id_source(format!("{} Tab Contents Resize Area", &main_window_title))
                    .default_size(egui_territory_rect.size())
                    .min_width(territory_settings.min_size.x
                    - 2.0 * territory_settings.inner_margins.x
                    - 1.0 * territory_settings.spacing)
                    .min_height(1.0)
                    .max_width(window.width()
                    - 2.0 * territory_settings.inner_margins.x
                    - 1.0 * territory_settings.spacing)
                    .max_height(window.height()
                    - 2.0 * territory_settings.inner_margins.x
                    - 1.0 * territory_settings.spacing);

                let is_resizing = matches!(territory_tabs_current_state.get(), TerritoryTabsState::ResizingTerritories);
                let is_dragging = matches!(territory_tabs_current_state.get(), TerritoryTabsState::DraggingTerritories);

                    

                egui::Window::new(&main_window_title)
                    .title_bar(false)
                    .frame(territory_frame)
                    .pivot(egui::Align2::CENTER_CENTER)
                    .min_width(1.0) // Doesn't appear to do anything??
                    .min_height(1.0) // Doesn't appear to do anything??
                    .max_width(window.width()) // Doesn't appear to do anything??
                    .max_height(window.height()) // Doesn't appear to do anything??
                    .default_size(egui_territory_rect.size())
                    .current_pos(egui_territory_rect.center())
                    .scroll2(false) // Doesn't appear to do anything??
                    .resizable(false)
                    .show(ctx, |ui| {

                        tab_contents_resize_area.show(ui, |ui| {
                            let bg_response = ui.vertical(|ui| {
                                tab_bar_scroll_area.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.add(tab_button).interact(egui::Sense::click_and_drag());
                                        //ui.add(tab_button).interact(egui::Sense::click_and_drag());
                                    });
                                });

                                ui.add(tab_border);

                                
                                tab_contents_scroll_area.show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.label("[Tab Contents Will Be Here]").highlight();
                                        
                                    });
                                
                                });

                                ui.allocate_space(ui.available_size());
                                ui.interact_bg(egui::Sense::click_and_drag());
                            }).response;

                            if !is_resizing {
                                // Drag detection setting off Territory drag logic.
                                if bg_response.drag_started() {
                                    territory_drag_started.send(TerritoryDragStarted);
                                }
                                if bg_response.dragged() {
                                    territory_dragged.send(
                                        TerritoryDragged { 
                                        window_entity,
                                        territory_entity,
                                        mouse_delta: Vec2::new(bg_response.drag_delta().x, - 1.0 * bg_response.drag_delta().y)
                                    });
                                }
                                if bg_response.drag_stopped() {
                                    territory_drag_ended.send(TerritoryDragEnded);
                                }
                            }

                            if !is_dragging {
                                // Resize detection setting off Territory resize logic.
                                let mouse_primary_down = ui.input(|i| i.pointer.primary_down());
                                let mouse_primary_released = ui.input(|i| i.pointer.primary_released());
                                let current_rect = egui::Rect::from_center_size(
                                    ui.min_rect().center(), 
                                    egui::Vec2::new(
                                        ui.min_rect().size().x, 
                                        ui.min_rect().size().y  
                                    )
                                );    

                                let mut delta_size = Vec2::new(
                                    current_rect.width() - egui_territory_rect.width(), 
                                    current_rect.height() - egui_territory_rect.height()
                                );

                                delta_size.x = f32::trunc(delta_size.x * 100.0) / 100.0;
                                delta_size.y = f32::trunc(delta_size.y * 100.0) / 100.0;

                                if !is_resizing && delta_size.abs().length() > 0.0 && mouse_primary_down {
                                    territory_resize_started.send(TerritoryResizeStarted);
                                }

                                if is_resizing && delta_size.abs().length() > 0.0 && mouse_primary_down {
                                    println!("{:?}", delta_size);
                                    territory_resized.send(
                                        TerritoryResizing {
                                            window_entity,
                                            territory_entity,
                                            delta_size
                                        });
                                } 

                                if is_resizing && mouse_primary_released {
                                    // One last resize event to clean up last delta before switching states.
                                    territory_resized.send(
                                        TerritoryResizing {
                                            window_entity,
                                            territory_entity,
                                            delta_size
                                        });
                                    territory_resize_ended.send(TerritoryResizeEnded);
                                }
                            }
                        }); 
                    }
                );
            }
        }
    }
}
