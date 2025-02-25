use crate::scenes::*;

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

    fn update(&mut self, network_manager: &mut NetworkManager) -> Option<MainLoopControl> {
        if network_manager.is_connected() && !self.nm_connected {
            self.nm_connected = true;
            network_manager.send_command(ServerAPICommand::Ping);
        }

        None
    }

    fn draw(&mut self) {
        match self.nav_sate {
            MenuNav::Primary => self.draw_primary_nav(),
            MenuNav::CreateRoom => self.draw_room_form(),
            MenuNav::JoinRoom => self.draw_room_list(),
        }

        if !self.nm_connected {
            self.draw_connecting_box();
        }
    }
}

impl MenuScene {
    fn draw_primary_nav(&mut self) {}

    fn draw_room_form(&mut self) {}

    fn draw_room_list(&mut self) {}

    fn draw_connecting_box(&mut self) {}
}
