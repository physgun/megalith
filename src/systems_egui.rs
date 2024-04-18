use bevy::prelude::*;
use bevy_egui::egui::epaint::Shadow;
use bevy_egui::egui::{Align2, Rect as eguiRect};
use bevy_egui::egui::{Pos2};
use bevy_egui::{egui, EguiContext};

use crate::components_ui::*;
use crate::input_manager::*;
use crate::resources_ui::*;
use crate::systems_ui::*;

// egui demands you give every egui window spawned a unique name.
// This resource exists to assign ascending integers to every egui window name request.

// Insert egui related resources.
pub fn initialize_egui_resources (mut commands: Commands) {
    
}

// How this shit work????
// Just gizmos for now!
pub fn display_placeholders(
    mut gizmos: Gizmos,
    placeholder_query: Query<&Placeholder>
) {
    for placeholder in & placeholder_query {
        gizmos.rect_2d(
            placeholder.visual_rects[0].center(), 
            0.0,
            placeholder.visual_rects[0].size(),
            Color::RED
        );
        gizmos.rect_2d(
            placeholder.visual_rects[1].center(), 
            0.0,
            placeholder.visual_rects[1].size(),
            Color::WHITE
        );
    }
}

// Toy function to get an idea of how this will all work.
// Try to see what egui can do as far as representing Territories goes.
pub fn egui_display_territories(
    mut gizmos: Gizmos,
    mut window_query: Query<(Entity, &Window, &mut EguiContext), With<EguiDisplay>>,
    territory_query: Query<(Entity, &Parent, &Territory)>
) {
    // Iterate through windows, and their Territory children with EguiDisplay components.
    for (window_entity, window, mut context) in &mut window_query {
        for (territory_entity_id, territory_parent, territory) in & territory_query {
            if territory_parent.get() == window_entity {
                // Both Bevy and egui have a Rect type in their libraries! Need to convert from Bevy to egui.
                let egui_territory_rect: eguiRect = eguiRect { 
                    min: Pos2::new(territory.rect.min.x, territory.rect.min.y), 
                    max: Pos2::new(territory.rect.max.x, territory.rect.max.y) 
                };

                // Convert bevy's Vec2 to egui's Pos2.
                let egui_territory_rect_center = Pos2::new(territory.rect.center().x, territory.rect.center().y);

                gizmos.rect_2d(
                    territory.rect.center(), 
                    0.0,
                    territory.rect.size(),
                    Color::BLUE
                );

                egui::Window::new(territory_entity_id.index().to_string())
                    .title_bar(false)
                    .frame(egui::Frame {
                        shadow: Shadow::NONE,
                        ..Default::default()
                    })
                    .pivot(Align2::CENTER_CENTER)
                    .default_rect(egui_territory_rect)
                    .default_pos(egui_territory_rect_center)
                    .show(context.get_mut(), |ui| {
                        ui.label("Blank Territory!");
                        ui.allocate_space(ui.available_size());
                        
                    }
                );
            }
        }
    }
}
