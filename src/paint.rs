use crate::{char_texture::CharTexture, texture_display::MainTexture, ui::UiContext};
use bevy::{prelude::*, render::camera::RenderTarget};

pub fn paint(
    sprites: Query<(&Transform, &Sprite)>,
    mut main_texture: ResMut<MainTexture>,
    mut ui_context: ResMut<UiContext>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: Res<Windows>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
) {
    ui_context.currently_painting = mouse_button_input.pressed(MouseButton::Left);
    if !ui_context.currently_painting || cursor_moved_events.is_empty() {
        if !ui_context.currently_painting {
            ui_context.last_paint_point = None;
        }
        return;
    }

    let sprite_x = main_texture.sprite_gen.char_texture.dimensions.0;
    let sprite_y = main_texture.sprite_gen.char_texture.dimensions.1;
    let sprite_loc = sprites.get_single().unwrap().0.translation;
    let sprite_size = sprites.get_single().unwrap().1.custom_size.unwrap();

    let mut previous_pixel = ui_context.last_paint_point;
    for event in cursor_moved_events.iter() {
        let mouse_loc = get_cursor_world_coordinates(event.position, &windows, &camera);
        //println!("mouse loc: [{},{}]",mouse_loc.x,mouse_loc.y);
        let relative_x = mouse_loc.x - sprite_loc.x + sprite_size.x / 2.;
        let relative_y = -(mouse_loc.y - sprite_loc.y - sprite_size.y / 2.);

        let current_pixel_x = relative_x / sprite_size.x * sprite_x as f32;
        let current_pixel_y = relative_y / sprite_size.y * sprite_y as f32;
        let current_pixel = Vec2::new(current_pixel_x, current_pixel_y);
        //println!("pixel loc: [{},{}]",current_pixel.x,current_pixel.y);

        paint_line(
            previous_pixel.unwrap_or(current_pixel),
            current_pixel,
            ui_context.paint_letter,
            ui_context.paint_radius as i32,
            &mut main_texture.sprite_gen.char_texture,
        );
        previous_pixel = Some(current_pixel);
    }

    ui_context.last_paint_point = previous_pixel;
}

fn paint_line(start: Vec2, end: Vec2, letter: char, radius: i32, texture: &mut CharTexture) {
    let step = (end - start).normalize_or_zero();
    let mut current_pixel = start;
    let mut last_distance_squared = f32::INFINITY;
    while current_pixel.distance_squared(end) < last_distance_squared {
        last_distance_squared = current_pixel.distance_squared(end);
        for x in -radius..radius {
            for y in -radius..radius {
                let circle_pixel: Vec2 =
                    (current_pixel.x + x as f32, current_pixel.y + y as f32).into();
                if current_pixel.distance_squared(circle_pixel) < radius.pow(2) as f32 {
                    let final_x = circle_pixel.x as i32;
                    let final_y = circle_pixel.y as i32;
                    if !texture.out_of_range(final_x, final_y) {
                        texture.set(final_x as usize, final_y as usize, letter);
                    }
                }
            }
        }
        current_pixel += step;
    }
}

// https://bevy-cheatbook.github.io/cookbook/cursor2world.html#convert-cursor-to-world-coordinates
fn get_cursor_world_coordinates(
    screen_pos: Vec2,
    // need to get window dimensions
    windows: &Res<Windows>,
    // query to get camera transform
    q_camera: &Query<(&Camera, &GlobalTransform), With<Camera>>,
) -> Vec2 {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // get the window that the camera is displaying to
    let window_id = match camera.target {
        RenderTarget::Window(id) => id,
        _ => panic!("main camera not projecting to a window"),
    };
    let wnd = windows.get(window_id).unwrap();

    // get the size of the window
    let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
    // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
    let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
    // matrix for undoing the projection and camera transform
    let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();
    // use it to convert ndc to world-space coordinates
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
    // reduce it to a 2D value
    world_pos.truncate()
}
