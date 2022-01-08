mod char_texture;
mod sprite_gen;
mod random;
mod noise;

use crate::sprite_gen::*;


use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::{camera::Camera, texture::*},
};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        //.add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(setup.system())
        .add_startup_system(texture_setup.system())
        .add_system(movement.system())
        .add_system(texture_update.system())
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    // camera
    //commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // commands.spawn_bundle(SpriteBundle {
    //     material: materials.add(ColorMaterial::color(Color::rgb(0.9, 0.9, 0.9))),
    //     transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
    //     sprite: Sprite::new(Vec2::new(50., 50.)),
    //     ..Default::default()
    // });
}

fn movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    let speed = 2000. * time.delta_seconds();
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

    if let Ok(mut transform) = camera.single_mut() {
        transform.translation += movement;
    } else {
        panic!("CAMERA NOT FOUND");
    }
}


// update resource
fn texture_update(
    keyboard_input: Res<Input<KeyCode>>,
    mut textures: ResMut<Assets<Texture>>,
    texture: Res<Handle<Texture>>,
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
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let width: usize = 256;
    let height: usize = 256;

    let mut sprite_gen = SpriteGen::new(width, height);
    let mut texture = vec![255u8;width*height*4];
    sprite_gen.randomize();
    sprite_gen.update_texture(&mut texture);

    for rule in sprite_gen.rules.iter() {
        println!("condition {} action {:?}", rule.condition.to_string(), rule.action);
    }
    for (char,color) in sprite_gen.char_color.iter() {
        println!("color for \"{}\": {:?}",char, color);
    }

    commands.insert_resource(sprite_gen);

    let texture_handle = textures.add(Texture::new_fill(
        Extent3d::new(width as u32, height as u32, 1),
        TextureDimension::D2,
        &texture,
        TextureFormat::Rgba8UnormSrgb,
    ));

    let material_handle = materials.add(ColorMaterial::texture(texture_handle.clone()));

    commands.spawn_bundle(SpriteBundle {
        material: material_handle,
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        sprite: Sprite::new(Vec2::new(800., 800.)),
        ..Default::default()
    });

    commands.insert_resource(texture_handle);
}

// fn spawn_sprites(
//     mut commands: Commands,
//     mut textures: ResMut<Assets<Texture>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
// ) {
//     let width: usize = 256;
//     let height: usize = 256;
//     let mut texture = PixelTexture::new(width,height);

//     let mut rng = rand::thread_rng();
//     let distr = rand::distributions::Uniform::new_inclusive(0, 255);

//     for x in 0..width {
//         for y in 0..height {
//             let pixel = [rng.sample(distr),rng.sample(distr),rng.sample(distr),rng.sample(distr)];
//             texture.set(x,y,pixel);
//         }
//     }
     
//     let texture_handle = textures.add(Texture::new_fill(
//         Extent3d::new(width as u32, height as u32, 1),
//         TextureDimension::D2,
//         texture.get(),
//         TextureFormat::Rgba8UnormSrgb,
//     ));

//     let material_handle = materials.add(ColorMaterial::texture(texture_handle));

//     commands.spawn_bundle(SpriteBundle {
//         material: material_handle,
//         transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
//         sprite: Sprite::new(Vec2::new(800., 800.)),
//         ..Default::default()
//     });
// }