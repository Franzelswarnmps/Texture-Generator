use crate::sprite_gen::*;
use crate::ui::*;

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub struct MainTexture {
    pub sprite_gen: SpriteGen,
    pub texture_handle: Handle<Image>,
}

impl MainTexture {
    pub fn new(sprite_gen: SpriteGen, texture_handle: Handle<Image>) -> Self {
        Self {
            sprite_gen,
            texture_handle,
        }
    }
}

pub fn refresh_texture(mut textures: ResMut<Assets<Image>>, mut main_texture: ResMut<MainTexture>) {
    if main_texture.sprite_gen.is_changed() {
        let texture = textures
            .get_mut(main_texture.texture_handle.clone())
            .unwrap();
        main_texture.sprite_gen.update_texture(&mut texture.data);
        main_texture.sprite_gen.set_changed();
    }
}

pub fn resize_texture(
    mut commands: Commands,
    sprites: Query<Entity, With<Sprite>>,
    mut textures: ResMut<Assets<Image>>,
    mut main_texture: ResMut<MainTexture>,
    mut ui_context: ResMut<UiContext>,
) {
    let new_width = ui_context.texture_dimensions.0;
    let new_height = ui_context.texture_dimensions.1;
    let old_width = main_texture.sprite_gen.char_texture.dimensions.0;
    let old_height = main_texture.sprite_gen.char_texture.dimensions.1;
    let size_changed = new_width != old_width || new_height != old_height;

    if ui_context.update_texture_dimensions {
        ui_context.update_texture_dimensions = false;
        if size_changed {
            // remove old
            for sprite in sprites.iter() {
                commands.entity(sprite).despawn();
            }
            textures.remove(main_texture.texture_handle.clone());

            // resize char_texture
            main_texture
                .sprite_gen
                .char_texture
                .resize(new_width, new_height);
            // create new texture
            let texture_handle = create_texture(&mut commands, textures, new_width, new_height);
            main_texture.texture_handle = texture_handle;
            main_texture.sprite_gen.set_changed(); // force refresh
        }
    }
}

pub fn create_texture(
    commands: &mut Commands,
    mut textures: ResMut<Assets<Image>>,
    width: usize,
    height: usize,
) -> Handle<Image> {
    let texture_handle = textures.add(Image::new_fill(
        Extent3d {
            width: width as u32,
            height: height as u32,
            ..Default::default()
        },
        TextureDimension::D2,
        &vec![255u8; width * height * 4],
        TextureFormat::Rgba8UnormSrgb,
    ));

    let custom_size = Vec2::new((width * 3) as f32, (height * 3) as f32);
    commands.spawn_bundle(SpriteBundle {
        texture: texture_handle.clone(),
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        sprite: Sprite {
            custom_size: Some(custom_size),
            ..Default::default()
        },
        ..Default::default()
    });

    texture_handle
}
