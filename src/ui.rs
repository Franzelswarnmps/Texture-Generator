use std::{time::Instant, io::Cursor};

use crate::{noise::noise_fill, rule::Rule, MainTexture, sprite_gen::SpriteGen};
use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_egui::{
    egui::{self, color::Hsva, Align2, ScrollArea, Slider},
    EguiContext,
};

pub struct UiContext {
    pub paint_letter: char,
    pub paint_radius: u8,
    pub currently_painting: bool,
    pub last_paint_point: Option<Vec2>,

    pub delay: u64,
    pub delay_start: Instant,

    pub texture_dimensions: (usize,usize),
    pub update_texture_dimensions: bool,

    pub saved_image: String,
}

impl UiContext {
    pub fn new() -> Self {
        Self {
            paint_letter: 'A',
            paint_radius: 10,
            currently_painting: false,
            last_paint_point: None,
            texture_dimensions: (0,0),
            update_texture_dimensions: false,
            delay: 0,
            delay_start: Instant::now(),
            saved_image: "".into(),
        }
    }
}

pub fn keybinds(
    mut egui_ctx: ResMut<EguiContext>,
    mut ui_context: ResMut<UiContext>,
    keyboard_input: Res<Input<KeyCode>>,
    mut scroll_evr: EventReader<MouseWheel>,
    mut camera: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    mut main_texture: ResMut<MainTexture>,
) {
    if egui_ctx.ctx_mut().wants_keyboard_input() 
    || egui_ctx.ctx_mut().wants_pointer_input() 
    || egui_ctx.ctx_mut().is_using_pointer() 
    || egui_ctx.ctx_mut().is_pointer_over_area() {
        return;
    }

    // movement
    let speed = 5.;
    let mut movement = Vec3::new(0., 0., 0.);
    if keyboard_input.pressed(KeyCode::W) {
        movement += Vec3::new(0., 1., 0.);
    }
    if keyboard_input.pressed(KeyCode::S) {
        movement += Vec3::new(0., -1., 0.);
    }
    if keyboard_input.pressed(KeyCode::A) {
        movement += Vec3::new(-1., 0., 0.);
    }
    if keyboard_input.pressed(KeyCode::D) {
        movement += Vec3::new(1., 0., 0.);
    }
    movement = movement.normalize_or_zero() * speed;
    camera.single_mut().0.translation += movement;

    // scroll
    let mut total_x = 0.;
    let mut total_y = 0.;
    for ev in scroll_evr.iter() {
        total_x += ev.x;
        total_y += ev.y;
    }
    let total_mouse_scroll = total_x + total_y;
    let current_scale = &mut camera.single_mut().1.scale;
    if keyboard_input.pressed(KeyCode::Equals) || total_mouse_scroll > 0. {
        *current_scale = (*current_scale - 0.1).clamp(0.0001, f32::INFINITY);
    }
    if keyboard_input.pressed(KeyCode::Minus) || total_mouse_scroll < 0. {
        *current_scale = (*current_scale + 0.1).clamp(0.0001, f32::INFINITY);
    }

    //actions

    if keyboard_input.pressed(KeyCode::Space) {
        run(&mut ui_context,&mut main_texture.sprite_gen);
    }

    if keyboard_input.just_pressed(KeyCode::C) {
        main_texture.sprite_gen.randomize_color();
        main_texture.sprite_gen.set_changed();
    }

    if keyboard_input.just_pressed(KeyCode::R) {
        main_texture.sprite_gen.randomize();
    }

    if keyboard_input.just_pressed(KeyCode::F) {
        main_texture.sprite_gen.randomize_rules();
    }

    if keyboard_input.just_pressed(KeyCode::I) {
        noise_fill(&mut main_texture.sprite_gen);
    }
}

pub fn egui(
    mut egui_ctx: ResMut<EguiContext>,
    mut ui_context: ResMut<UiContext>,
    mut main_texture: ResMut<MainTexture>,
) {
    let sprite_gen = &mut main_texture.sprite_gen;
    egui::Window::new("Rules [Conidition, Action]")
        .anchor(Align2::RIGHT_TOP, [0., 0.])
        .min_width(200.)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                if ui.button("+").clicked() {
                    sprite_gen.rules.push(Rule::new("", ""));
                }
                if ui.button("-").clicked() {
                    sprite_gen.rules.pop();
                }
            });

            ui.columns(2, |ui| {
                for rule in sprite_gen.rules.iter_mut() {
                    let mut current_condition = rule.original_condition().to_owned();
                    ui[0].text_edit_singleline(&mut current_condition);
                    if current_condition != *rule.original_condition() {
                        rule.set_condition(&current_condition);
                    }
                }
                for rule in sprite_gen.rules.iter_mut() {
                    let mut current_action = rule.original_action().to_owned();
                    ui[1].text_edit_singleline(&mut current_action);
                    if current_action != *rule.original_action() {
                        rule.set_action(&current_action);
                    }
                }
            });
        });

    egui::SidePanel::left("side_panel2")
        .default_width(140.0)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    let width = sprite_gen.char_texture.dimensions.0;
                    let height = sprite_gen.char_texture.dimensions.1;
                    let mut data = vec![255u8; width * height * 4];
                    sprite_gen.update_texture(&mut data);

                    let mut output: Vec<u8> = Vec::new();
                    let mut cursor = Cursor::new(&mut output);
                    if image::write_buffer_with_format(
                        &mut cursor,
                        &data,
                        width.try_into().unwrap(),
                        height.try_into().unwrap(),
                        image::ColorType::Rgba8,
                        image::ImageFormat::Png
                    ).is_err() {
                        println!("failed to save image");
                    };

                    ui_context.saved_image = format!("{}{}","data:image/png;base64,",base64::encode(output));
                }
                ui.text_edit_singleline(&mut ui_context.saved_image);
            });

            ui.separator();

            if ui.button("Export Rules & Colors").clicked() {
                println!("random");
            }
            if ui.button("Import Rules & Colors").clicked() {
                println!("random");
            }

            ui.separator();


            if ui.button("Randomize All (R)").clicked() {
                sprite_gen.randomize();
            }
            if ui.button("Randomize Colors (C)").clicked() {
                sprite_gen.randomize_color();
            }
            if ui.button("Randomize Image (I)").clicked() {
                noise_fill(sprite_gen);
            }
            if ui.button("Randomize Rules (F)").clicked() {
                sprite_gen.randomize_rules();
            }
            if ui.button("Run (Space)").clicked() {
                run(&mut ui_context,sprite_gen);
            }

            ui.separator();

            ui.horizontal(|ui| {
                ui.set_max_width(100.);

                if ui.button("Apply").clicked() {
                    ui_context.update_texture_dimensions = true;
                }

                ui.horizontal(|ui| {
                    ui.set_max_width(60.);
                    ui.label("W");
                    let original_width: String = ui_context.texture_dimensions.0.to_string();
                    let mut width: String = original_width.clone();
                    ui.text_edit_singleline(&mut width);
                    if width != original_width {
                        ui_context.texture_dimensions.0 = width.parse().unwrap_or(0);
                    }
                });

                ui.horizontal(|ui| {
                    ui.set_max_width(60.);
                    ui.label("H");
                    let original_height: String = ui_context.texture_dimensions.1.to_string();
                    let mut height: String = original_height.clone();
                    ui.text_edit_singleline(&mut height);
                    if height != original_height {
                        ui_context.texture_dimensions.1 = height.parse().unwrap_or(0);
                    }
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Delay");
                ui.add(Slider::new(&mut ui_context.delay, 0..=1000));
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Paint");
                ui.add(Slider::new(&mut ui_context.paint_radius, 1..=30));
            });

            /* colors  */
            ui.horizontal(|ui| {
                if ui.button("+").clicked() {
                    let last_letter = sprite_gen
                        .char_color
                        .iter()
                        .map(|x| x.0.to_owned())
                        .last()
                        .unwrap_or('@');
                    let next_letter =
                        std::char::from_u32(last_letter as u32 + 1).unwrap_or(last_letter);
                    if next_letter.is_alphabetic() && last_letter != next_letter {
                        sprite_gen.char_color.insert(next_letter, [0, 0, 0, 0]);
                    }
                }
                if ui.button("-").clicked() {
                    if let Some(c) = sprite_gen.char_color.iter().map(|x| x.0.to_owned()).last() {
                        sprite_gen.char_color.remove(&c);
                    }
                }
            });

            let mut colors_changed = false;
            ScrollArea::vertical()
            .max_height(ui.available_height())
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        for (letter, color) in sprite_gen.char_color.iter_mut() {
                            let mut current_color = Hsva::from_srgb([color[0], color[1], color[2]]);
                            ui.horizontal(|ui| {
                                let letter_string = [*letter].iter().collect::<String>();
                                ui.label(letter_string);
                                ui.color_edit_button_hsva(&mut current_color);
                                if ui.radio(ui_context.paint_letter == *letter, "").clicked() {
                                    ui_context.paint_letter = *letter;
                                }
                                ui.label(format!("{}", sprite_gen.char_texture.count(*letter)));
                            });

                            let compare_color = current_color.to_srgb();
                            if color[0] != compare_color[0]
                                || color[1] != compare_color[1]
                                || color[2] != compare_color[2]
                            {
                                color[0] = compare_color[0];
                                color[1] = compare_color[1];
                                color[2] = compare_color[2];
                                colors_changed = true;
                            }
                        }

                        // artificial padding
                        ui.vertical(|ui| {
                            ui.set_height(2000.);
                        });
                    });
                });

            if colors_changed {
                sprite_gen.set_changed();
            }
        });
}

fn run(ui_context: &mut UiContext, sprite_gen: &mut SpriteGen) {
    if ui_context.delay < 1 {
        sprite_gen.apply();
    } else if ui_context.delay_start.elapsed().as_millis() >= ui_context.delay.into() {
        sprite_gen.apply();
        ui_context.delay_start = Instant::now();
    }
}