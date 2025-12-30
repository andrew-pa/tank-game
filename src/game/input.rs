use raylib::prelude::{KeyboardKey, MouseButton, RaylibHandle, Vector2};

#[derive(Clone, Copy, Debug)]
pub struct PlayerInput {
    pub turn: f32,
    pub movement: f32,
    pub aim_dir: Option<Vector2>,
    pub wants_fire: bool,
    pub use_mouse_aim: bool,
}

pub struct InputState;

impl InputState {
    pub fn new() -> Self {
        Self
    }

    pub fn player_input(&mut self, rl: &RaylibHandle) -> PlayerInput {
        let mut turn: f32 = 0.0;
        if rl.is_key_down(KeyboardKey::KEY_A) || rl.is_key_down(KeyboardKey::KEY_LEFT) {
            turn -= 1.0;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) || rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            turn += 1.0;
        }

        let mut movement: f32 = 0.0;
        if rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP) {
            movement += 1.0;
        }
        if rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN) {
            movement -= 1.0;
        }

        let wants_fire = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
            || rl.is_key_down(KeyboardKey::KEY_SPACE);

        PlayerInput {
            turn,
            movement,
            aim_dir: None,
            wants_fire,
            use_mouse_aim: true,
        }
    }
}
