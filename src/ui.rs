use crate::sprite_gen::*;
use bevy::prelude::*;
use bevy_egui::{egui::{self, color::Hsva}, EguiContext};//, EguiPlugin, EguiSettings};

pub fn ui_example(
    egui_ctx: ResMut<EguiContext>,
    mut sprite_gen: ResMut<SpriteGen>,
    mut textures: ResMut<Assets<Image>>,
    texture: Res<Handle<Image>>,
) {
    egui::SidePanel::left("side_panel1")
        .default_width(140.0)
        .show(egui_ctx.ctx(), |ui| {
            ui.heading("Conditions");

            for rule in sprite_gen.rules.iter_mut() {
                let mut current_condition = rule.original_condition().to_owned();
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut current_condition);
                });
                if current_condition != *rule.original_condition() {
                    rule.set_condition(&current_condition);
                }
            }

            let mut colors_changed = false;
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
                    colors_changed = true;
                }
            }

            if colors_changed {
                let texture = textures.get_mut(&*texture).unwrap();
                sprite_gen.update_texture(&mut texture.data);
            }
        });

    egui::SidePanel::left("side_panel2")
        .default_width(140.0)
        .show(egui_ctx.ctx(), |ui| {
            ui.heading("Actions");

            for rule in sprite_gen.rules.iter_mut() {
                let mut current_action = rule.original_action().to_owned();
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut current_action);
                });
                if current_action != *rule.original_action() {
                    rule.set_action(&current_action);
                }
            }
        });
}