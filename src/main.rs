mod char_texture;
mod sprite_gen;
mod random;
mod noise;
mod ui;

use crate::sprite_gen::*;
use crate::ui::*;

use bevy::{
    //diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::{camera::Camera, render_resource::{Extent3d, TextureDimension, TextureFormat}},
};

use bevy_egui::{EguiPlugin};


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //.add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EguiPlugin)
        .add_system(ui_example)
        .add_startup_system(setup)
        .add_startup_system(texture_setup)
        .add_system(movement)
        .add_system(texture_update)
        .run();
}

fn setup(mut commands: Commands) {
    // camera
    //commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn movement(
    //time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    let speed = 20.;// * time.delta_seconds();
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
    if keyboard_input.pressed(KeyCode::Space) {
        movement *= 4.;
    }

    camera.single_mut().translation += movement;
}


// update resource
fn texture_update(
    keyboard_input: Res<Input<KeyCode>>,
    mut textures: ResMut<Assets<Image>>,
    texture: Res<Handle<Image>>,
    mut sprite_gen: ResMut<SpriteGen>,
) {
    let texture = textures.get_mut(&*texture).unwrap();

    if keyboard_input.pressed(KeyCode::Space) {
        sprite_gen.apply();
    
        sprite_gen.update_texture(&mut texture.data);
    }

    if keyboard_input.just_pressed(KeyCode::C) {
        sprite_gen.randomize_color();
    
        sprite_gen.update_texture(&mut texture.data);
    }

    if keyboard_input.just_pressed(KeyCode::R) {
        sprite_gen.randomize();
        sprite_gen.update_texture(&mut texture.data);
    }
}

// make texture, add as resource. also add shrite_gen as resource
fn texture_setup(
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
) {
    let width: usize = 256;
    let height: usize = 256;

    let mut sprite_gen = SpriteGen::new(width, height);
    let mut texture = vec![255u8;width*height*4];
    sprite_gen.randomize();
    sprite_gen.update_texture(&mut texture);

    // for rule in sprite_gen.rules.iter() {
    //     println!("condition {} action {:?}", rule.condition.to_string(), rule.action);
    // }
    // for (char,color) in sprite_gen.char_color.iter() {
    //     println!("color for \"{}\": {:?}",char, color);
    // }

    commands.insert_resource(sprite_gen);

    let texture_handle = textures.add(Image::new_fill(
        Extent3d {
            width: width as u32,
            height: height as u32,
            ..Default::default()
        },
        TextureDimension::D2,
        &texture,
        TextureFormat::Rgba8UnormSrgb,
    ));

    commands.spawn_bundle(SpriteBundle {
        texture: texture_handle.clone(),
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        sprite: Sprite { 
            custom_size: Some(Vec2::new(800., 800.)),
            ..Default::default()
        },
        ..Default::default()
    });

    commands.insert_resource(texture_handle);
}