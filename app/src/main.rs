use raylib::prelude::*;
mod colors;
mod network;
mod scenes;
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

    let mut nm = network::NetworkManager::new("ws://127.0.0.1:8080/");

    let mut scenes: Vec<Box<dyn scenes::Scene>> = vec![Box::<scenes::MenuScene>::default()];
    let mut current_scene: usize = 0;
    scenes[0].on_enter();

    while !rl_handle.window_should_close() {
        nm.udpate();
        scenes[current_scene].update(&mut rl_handle, &mut nm);

        let mut rl_draw_handle = rl_handle.begin_drawing(&rl_thread);
        rl_draw_handle.clear_background(COLOR_DARK);

        scenes[current_scene].draw(&mut rl_draw_handle);
    }
}
