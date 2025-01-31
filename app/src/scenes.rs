use raylib::RaylibHandle;
use raylib::drawing::RaylibDrawHandle;

pub trait Scene {
    fn get_title(&self) -> &str;
    fn has_background(&self) -> bool;
    fn has_safe_exit(&self) -> bool;
    /// Update the scene (only logic)
    fn update(&mut self, rl_handle: &mut RaylibHandle);
    /// Draw one frame of the scene
    fn draw(&mut self, rl_handle: &mut RaylibDrawHandle);
}
