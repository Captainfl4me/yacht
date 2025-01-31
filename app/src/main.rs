use raylib::core::texture::Image;
use raylib::prelude::*;
use std::ffi::CStr;

mod scenes;

mod colors;
use colors::*;

fn main() {
    let (mut rl_handle, rl_thread) = raylib::init()
        .size(640, 480)
        .resizable()
        .title("Yatch")
        .build();
    rl_handle.set_target_fps(60);
    rl_handle.set_exit_key(None);
    rl_handle.gui_set_style(
        GuiControl::DEFAULT,
        GuiDefaultProperty::TEXT_SIZE as i32,
        18,
    );
    rl_handle.set_window_state(rl_handle.get_window_state().set_window_maximized(true));

    let image_bytes = include_bytes!("../assets/background_tile.png");
    let mut background_tile_image = Image::load_image_from_mem(".png", image_bytes).unwrap();
    background_tile_image.resize(256, 256);
    let background_tile_texture = rl_handle
        .load_texture_from_image(&rl_thread, &background_tile_image)
        .unwrap();

    let mut scenes: Vec<Box<dyn scenes::Scene>> = vec![];
    let mut current_scene: Option<usize> = None;

    const TITLE_FONT_SIZE: i32 = 80;
    while !rl_handle.window_should_close() {
        if let Some(scene_id) = current_scene {
            scenes[scene_id].update(&mut rl_handle);
        }

        if current_scene.is_some() && rl_handle.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            if scenes[current_scene.unwrap()].has_safe_exit() {
                println!("Exit prevented!!"); // #TODO
            } else {
                current_scene = None;
            }
        }

        // Draw frame
        {
            let mut rl_draw_handle = rl_handle.begin_drawing(&rl_thread);
            let screen_width = rl_draw_handle.get_screen_width();
            let screen_height = rl_draw_handle.get_screen_height();
            rl_draw_handle.clear_background(COLOR_DARK);
            if current_scene.is_none() || !scenes[current_scene.unwrap()].has_background() {
                draw_background(&mut rl_draw_handle, &background_tile_texture);
            }

            if let Some(scene_id) = current_scene {
                scenes[scene_id].draw(&mut rl_draw_handle);
            } else {
                rl_draw_handle.draw_text(
                    "YATCH",
                    (screen_width - rl_draw_handle.measure_text("YATCH", TITLE_FONT_SIZE)) / 2,
                    (screen_height - TITLE_FONT_SIZE) / 2,
                    TITLE_FONT_SIZE,
                    COLOR_LIGHT,
                );

                if !scenes.is_empty() {
                    let load_scenes_text_list = scenes
                        .iter()
                        .map(|s| format!("Load: {}", s.get_title()))
                        .collect::<Vec<_>>();
                    let max_button_width = load_scenes_text_list
                        .iter()
                        .map(|text| rl_draw_handle.measure_text(text, 30))
                        .max()
                        .unwrap();

                    for (i, mut scene_name) in load_scenes_text_list.into_iter().enumerate() {
                        scene_name.push('\0');
                        if rl_draw_handle.gui_button(
                            Rectangle::new(
                                (screen_width - max_button_width) as f32 / 2.0,
                                (screen_height + TITLE_FONT_SIZE) as f32 / 2.0
                                    + 10.0
                                    + (40 * (i as i32)) as f32,
                                max_button_width as f32 + 20.0,
                                30.0,
                            ),
                            Some(CStr::from_bytes_with_nul(scene_name.as_bytes()).unwrap()),
                        ) {
                            current_scene = Some(i);
                        }
                    }
                }
            }
        }
    }
}

fn draw_background(rl_draw_handle: &mut RaylibDrawHandle, tile_texture: &Texture2D) {
    let screen_width = rl_draw_handle.get_screen_width();
    let screen_height = rl_draw_handle.get_screen_height();
    for i in 0..=screen_width / tile_texture.width() {
        for j in 0..=screen_height / tile_texture.height() {
            rl_draw_handle.draw_texture(
                tile_texture,
                i * tile_texture.width(),
                j * tile_texture.height(),
                COLOR_LIGHT,
            );
        }
    }
    rl_draw_handle.draw_text(
        "CREDITS: Captainfl4me",
        screen_width - rl_draw_handle.measure_text("CREDITS: Captainfl4me", 32) - 20,
        screen_height - 40,
        32,
        COLOR_BLACK,
    );
}
