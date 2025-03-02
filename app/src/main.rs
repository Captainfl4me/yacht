use macroquad::prelude::*;
mod colors;
mod network;
mod scenes;
use colors::*;

#[macroquad::main("Yatch")]
async fn main() {
    let mut nm = network::NetworkManager::new("ws://127.0.0.1:8080/");

    let mut scenes: Vec<Box<dyn scenes::Scene>> = vec![Box::<scenes::MenuScene>::default()];
    let mut current_scene: usize = 0;
    scenes[0].on_enter();

    loop {
        nm.udpate();
        scenes[current_scene].update(&mut nm);

        clear_background(BLACK);
        scenes[current_scene].draw();

        draw_circle(mouse_position().0, mouse_position().1, 10.0, RED);

        next_frame().await;
    }
}
