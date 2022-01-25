use crate::sprite_gen::*;
use bevy::prelude::*;
use bevy_egui::{egui::{self, color::Hsva}, EguiContext, EguiPlugin, EguiSettings};

pub fn ui_example(
    mut egui_ctx: ResMut<EguiContext>,
    mut sprite_gen: ResMut<SpriteGen>,
) {
    egui::SidePanel::left("side_panel1")
        .default_width(140.0)
        .show(egui_ctx.ctx(), |ui| {
            ui.heading("Conditions");

            for rule in sprite_gen.rules.iter_mut() {
                let mut current_condition = rule.get_condition().to_owned();
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut current_condition);
                });
                if current_condition != *rule.get_condition() {
                    rule.update_condition(current_condition);
                }
            }

            for (letter, color) in sprite_gen.char_color.iter_mut() {
                let mut current_color = Hsva::from_srgb([color[0],color[1],color[2]]);
                ui.horizontal(|ui| {
                    ui.label([*letter].iter().collect::<String>());
                    ui.color_edit_button_hsva(&mut current_color)
                });

                let compare_color = current_color.to_srgb();
                if 
                color[0] != compare_color[0] || 
                color[1] != compare_color[1] || 
                color[2] != compare_color[2] {
                    color[0] = compare_color[0];
                    color[1] = compare_color[1];
                    color[2] = compare_color[2];
                }
            }

        });

    egui::SidePanel::left("side_panel2")
        .default_width(140.0)
        .show(egui_ctx.ctx(), |ui| {
            ui.heading("Actions");

            for rule in sprite_gen.rules.iter_mut() {
                let mut current_action = rule.get_action().to_owned();
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut current_action);
                });
                if current_action != *rule.get_action() {
                    rule.update_action(current_action);
                }
            }
        });
}