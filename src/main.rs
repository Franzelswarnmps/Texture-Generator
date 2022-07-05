mod char_texture;
mod paint;
mod random_rules;
mod rule;
mod save_and_load;
mod sprite_gen;
mod texture_display;
mod texture_noise;
mod ui;

use crate::paint::*;
use crate::sprite_gen::*;
use crate::texture_display::*;
use crate::ui::*;

use bevy::prelude::*;

use bevy_egui::EguiPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .add_system(egui)
        .add_system(keybinds)
        .add_system(paint)
        .add_system(refresh_texture)
        .add_system(resize_texture);

    #[cfg(target_family = "wasm")]
    app.add_plugin(bevy_web_fullscreen::FullViewportPlugin);

    #[cfg(debug_assertions)]
    {
        app.add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
            .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default());
    }

    app.run();
}

fn setup(mut commands: Commands, textures: ResMut<Assets<Image>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let default_width = 256;
    let default_height = 256;

    let mut ui_context = UiContext::new();
    ui_context.texture_dimensions.0 = default_width;
    ui_context.texture_dimensions.1 = default_height;
    commands.insert_resource(ui_context);

    let mut sprite_gen = SpriteGen::new(default_width, default_height);
    sprite_gen.randomize();
    let texture_handle = create_texture(&mut commands, textures, default_width, default_height);
    let main_texture = MainTexture::new(sprite_gen, texture_handle);
    commands.insert_resource(main_texture);
}
