use raylib::prelude::{
    GamepadAxis, GamepadButton, KeyboardKey, MouseButton, RaylibHandle, Vector2,
};

use crate::math::{vec2, vec2_length, vec2_normalize, vec2_scale};

const GAMEPAD_ID: i32 = 0;
const STICK_DEADZONE: f32 = 0.2;
const AIM_DEADZONE: f32 = 0.25;
const TRIGGER_THRESHOLD: f32 = 0.4;
const TRIGGER_ACTIVE: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputDevice {
    KeyboardMouse,
    Gamepad,
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerInput {
    pub turn: f32,
    pub movement: f32,
    pub aim_dir: Option<Vector2>,
    pub wants_fire: bool,
    pub use_mouse_aim: bool,
}

pub struct InputState {
    last_device: InputDevice,
    gamepad_available: bool,
    gamepad_id: i32,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            last_device: InputDevice::KeyboardMouse,
            gamepad_available: false,
            gamepad_id: GAMEPAD_ID,
        }
    }

    pub fn last_device(&self) -> InputDevice {
        self.last_device
    }

    pub fn gamepad_available(&self) -> bool {
        self.gamepad_available
    }

    pub fn start_pressed(&mut self, rl: &RaylibHandle) -> bool {
        self.refresh_gamepad(rl);
        let mut pressed = false;
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER)
            || rl.is_key_pressed(KeyboardKey::KEY_SPACE)
        {
            self.last_device = InputDevice::KeyboardMouse;
            pressed = true;
        }

        if self.gamepad_available
            && (rl.is_gamepad_button_pressed(
                self.gamepad_id,
                GamepadButton::GAMEPAD_BUTTON_MIDDLE_RIGHT,
            ) || rl.is_gamepad_button_pressed(
                self.gamepad_id,
                GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN,
            ))
        {
            self.last_device = InputDevice::Gamepad;
            pressed = true;
        }
        pressed
    }

    pub fn player_input(&mut self, rl: &RaylibHandle) -> PlayerInput {
        self.refresh_gamepad(rl);
        let keyboard = sample_keyboard_mouse(rl);
        let gamepad = if self.gamepad_available {
            sample_gamepad(rl, self.gamepad_id)
        } else {
            GamepadSample::default()
        };

        if gamepad.active && !keyboard.active {
            self.last_device = InputDevice::Gamepad;
        } else if keyboard.active && !gamepad.active {
            self.last_device = InputDevice::KeyboardMouse;
        }

        let turn = pick_axis(self.last_device, keyboard.turn, gamepad.turn);
        let movement = pick_axis(self.last_device, keyboard.movement, gamepad.movement);
        let aim_dir = if self.last_device == InputDevice::Gamepad {
            gamepad.aim_dir
        } else {
            None
        };
        let wants_fire = keyboard.wants_fire || gamepad.wants_fire;

        PlayerInput {
            turn,
            movement,
            aim_dir,
            wants_fire,
            use_mouse_aim: self.last_device == InputDevice::KeyboardMouse,
        }
    }

    fn refresh_gamepad(&mut self, rl: &RaylibHandle) {
        self.gamepad_available = rl.is_gamepad_available(self.gamepad_id);
        if !self.gamepad_available && self.last_device == InputDevice::Gamepad {
            self.last_device = InputDevice::KeyboardMouse;
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct KeyboardMouseSample {
    turn: f32,
    movement: f32,
    wants_fire: bool,
    active: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct GamepadSample {
    turn: f32,
    movement: f32,
    aim_dir: Option<Vector2>,
    wants_fire: bool,
    active: bool,
}

fn sample_keyboard_mouse(rl: &RaylibHandle) -> KeyboardMouseSample {
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

    let mouse_delta = rl.get_mouse_delta();
    let mouse_moved = mouse_delta.x.abs() > 0.0 || mouse_delta.y.abs() > 0.0;
    let wants_fire = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
        || rl.is_key_down(KeyboardKey::KEY_SPACE);
    let active = turn.abs() > 0.01 || movement.abs() > 0.01 || wants_fire || mouse_moved;

    KeyboardMouseSample {
        turn,
        movement,
        wants_fire,
        active,
    }
}

fn sample_gamepad(rl: &RaylibHandle, gamepad: i32) -> GamepadSample {
    let left_raw = vec2(
        rl.get_gamepad_axis_movement(gamepad, GamepadAxis::GAMEPAD_AXIS_LEFT_X),
        rl.get_gamepad_axis_movement(gamepad, GamepadAxis::GAMEPAD_AXIS_LEFT_Y),
    );
    let right_raw = vec2(
        rl.get_gamepad_axis_movement(gamepad, GamepadAxis::GAMEPAD_AXIS_RIGHT_X),
        rl.get_gamepad_axis_movement(gamepad, GamepadAxis::GAMEPAD_AXIS_RIGHT_Y),
    );

    let left = apply_radial_deadzone(left_raw, STICK_DEADZONE);
    let right = apply_radial_deadzone(right_raw, AIM_DEADZONE);

    let dpad_turn = if rl.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT) {
        -1.0
    } else if rl.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT) {
        1.0
    } else {
        0.0
    };
    let dpad_move = if rl.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP) {
        1.0
    } else if rl.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN) {
        -1.0
    } else {
        0.0
    };

    let turn = (left.x + dpad_turn).clamp(-1.0, 1.0);
    let movement = (-left.y + dpad_move).clamp(-1.0, 1.0);

    let aim_dir = if vec2_length(right) > 0.01 { Some(right) } else { None };

    let right_trigger = normalize_trigger(rl.get_gamepad_axis_movement(
        gamepad,
        GamepadAxis::GAMEPAD_AXIS_RIGHT_TRIGGER,
    ));
    let wants_fire = right_trigger > TRIGGER_THRESHOLD
        || rl.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_1)
        || rl.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_2)
        || rl.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN);

    let active = vec2_length(left_raw) > STICK_DEADZONE
        || vec2_length(right_raw) > AIM_DEADZONE
        || right_trigger > TRIGGER_ACTIVE
        || rl.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_1)
        || rl.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_2)
        || rl.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN)
        || rl.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT)
        || rl.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT)
        || rl.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP)
        || rl.is_gamepad_button_down(gamepad, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN);

    GamepadSample {
        turn,
        movement,
        aim_dir,
        wants_fire,
        active,
    }
}

fn apply_radial_deadzone(value: Vector2, deadzone: f32) -> Vector2 {
    let len = vec2_length(value);
    if len <= deadzone {
        vec2(0.0, 0.0)
    } else {
        let scaled = (len - deadzone) / (1.0 - deadzone);
        vec2_scale(vec2_normalize(value), scaled)
    }
}

fn normalize_trigger(axis: f32) -> f32 {
    if axis < -0.2 {
        ((axis + 1.0) * 0.5).clamp(0.0, 1.0)
    } else {
        axis.clamp(0.0, 1.0)
    }
}

fn pick_axis(device: InputDevice, keyboard: f32, gamepad: f32) -> f32 {
    match device {
        InputDevice::KeyboardMouse => {
            if keyboard.abs() > 0.01 {
                keyboard
            } else {
                gamepad
            }
        }
        InputDevice::Gamepad => {
            if gamepad.abs() > 0.01 {
                gamepad
            } else {
                keyboard
            }
        }
    }
}
