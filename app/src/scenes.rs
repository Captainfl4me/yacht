use raylib::RaylibHandle;
use raylib::drawing::RaylibDrawHandle;
use crate::network::NetworkManager;

pub use raylib::prelude::*;
pub use crate::colors::*;
pub use crate::network::*;

mod menu;
pub use menu::MenuScene;
mod game;

pub enum MainLoopControl {
    ChangeScene(usize),
}

#[derive(Clone, Copy)]
pub enum SceneLoadingState {
    Entering,
    Playing,
    Closing,
    Unload,
}

pub trait Scene {
    /// Retrieve SceneLoadingState from current scene.
    fn get_loading_state(&self) -> SceneLoadingState;
    /// Call only once before .update() on the scene entering.
    fn on_enter(&mut self);
    /// Call only once after .update() on the scene entering.
    fn on_close(&mut self);
    /// Update the scene (only logic)
    fn update(&mut self, rl_handle: &mut RaylibHandle, network_manager: &mut NetworkManager) -> Option<MainLoopControl>;
    /// Draw one frame of the scene
    fn draw(&mut self, rl_handle: &mut RaylibDrawHandle);
}
