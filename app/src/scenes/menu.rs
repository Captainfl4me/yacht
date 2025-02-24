use crate::scenes::*;
use std::ffi::CStr;

enum MenuNav {
    Primary,
    CreateRoom,
    JoinRoom,
}

pub struct MenuScene {
    state: SceneLoadingState,
    nm_connected: bool,
    nav_sate: MenuNav,
}
impl Default for MenuScene {
    fn default() -> Self {
        MenuScene {
            state: SceneLoadingState::Unload,
            nm_connected: false,
            nav_sate: MenuNav::Primary,
        }
    }
}

impl Scene for MenuScene {
    fn get_loading_state(&self) -> SceneLoadingState {
        self.state
    }

    fn on_enter(&mut self) {
        self.nm_connected = false;
        self.state = SceneLoadingState::Playing;
        self.nav_sate = MenuNav::Primary;
    }

    fn on_close(&mut self) {
        self.nm_connected = false;
        self.state = SceneLoadingState::Unload;
    }

    fn update(
        &mut self,
        rl_handle: &mut RaylibHandle,
        network_manager: &mut NetworkManager,
    ) -> Option<MainLoopControl> {
        if network_manager.is_connected() && !self.nm_connected {
            self.nm_connected = true;
            network_manager.send_command(ServerAPICommand::Ping);
        }

        None
    }

    fn draw(&mut self, rl_handle: &mut RaylibDrawHandle) {
        let screen_width = rl_handle.get_screen_width();
        let screen_height = rl_handle.get_screen_height();

        rl_handle.draw_text(
            "YATCH",
            (screen_width - rl_handle.measure_text("YATCH", 64)) / 2,
            64,
            64,
            COLOR_LIGHT,
        );

        rl_handle.draw_circle(
            rl_handle.get_mouse_x(),
            rl_handle.get_mouse_y(),
            10.0,
            COLOR_RED,
        );
        println!("x: {}", rl_handle.get_mouse_x());

        match self.nav_sate {
            MenuNav::Primary => self.draw_primary_nav(rl_handle, screen_width, screen_height),
            MenuNav::CreateRoom => self.draw_room_form(rl_handle, screen_width, screen_height),
            MenuNav::JoinRoom => self.draw_room_list(rl_handle, screen_width, screen_height),
        }

        if !self.nm_connected {
            self.draw_connecting_box(rl_handle, screen_width, screen_height);
        }
    }
}

impl MenuScene {
    fn draw_primary_nav(
        &mut self,
        rl_handle: &mut RaylibDrawHandle,
        screen_width: i32,
        screen_height: i32,
    ) {
        let max_button_width = rl_handle.measure_text("create room", 32);
        if rl_handle.gui_button(
            Rectangle::new(
                (screen_width - max_button_width) as f32 / 2.0,
                (screen_height - 40) as f32 / 2.0 + 50.0,
                max_button_width as f32 + 20.0,
                32.0,
            ),
            Some(CStr::from_bytes_with_nul(b"create room\0").unwrap()),
        ) {
            println!("hi there!");
            self.nav_sate = MenuNav::CreateRoom;
        }

        if rl_handle.gui_button(
            Rectangle::new(
                (screen_width - max_button_width) as f32 / 2.0,
                (screen_height + 40) as f32 / 2.0 + 50.0,
                max_button_width as f32 + 20.0,
                32.0,
            ),
            Some(CStr::from_bytes_with_nul(b"join room\0").unwrap()),
        ) {
            self.nav_sate = MenuNav::JoinRoom;
        }
    }

    fn draw_room_form(
        &mut self,
        rl_handle: &mut RaylibDrawHandle,
        screen_width: i32,
        screen_height: i32,
    ) {
        if rl_handle.gui_button(
            Rectangle::new(
                100.0,
                100.0,
                rl_handle.measure_text("back", 32) as f32 + 20.0,
                32.0,
            ),
            Some(CStr::from_bytes_with_nul(b"back\0").unwrap()),
        ) {
            self.nav_sate = MenuNav::Primary;
        }
    }

    fn draw_room_list(
        &mut self,
        rl_handle: &mut RaylibDrawHandle,
        screen_width: i32,
        screen_height: i32,
    ) {
        if rl_handle.gui_button(
            Rectangle::new(
                100.0,
                100.0,
                rl_handle.measure_text("back", 32) as f32 + 20.0,
                32.0,
            ),
            Some(CStr::from_bytes_with_nul(b"back\0").unwrap()),
        ) {
            self.nav_sate = MenuNav::Primary;
        }
    }

    fn draw_connecting_box(
        &mut self,
        rl_handle: &mut RaylibDrawHandle,
        screen_width: i32,
        screen_height: i32,
    ) {
        let panel_width: i32 = 600;
        let panel_height: i32 = 100;
        let panel_x = (screen_width - panel_width) / 2;
        let panel_y = (screen_height - panel_height) / 2;

        rl_handle.draw_rectangle_rounded(
            Rectangle::new(
                panel_x as f32,
                panel_y as f32,
                panel_width as f32,
                panel_height as f32,
            ),
            0.1,
            20,
            COLOR_DARK,
        );

        rl_handle.draw_text(
            "Connecting...",
            panel_x + (panel_width - rl_handle.measure_text("Connecting...", 32)) / 2,
            panel_y + 16,
            32,
            COLOR_LIGHT,
        );
    }
}
