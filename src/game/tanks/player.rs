use raylib::prelude::{KeyboardKey, MouseButton, RaylibHandle, Vector2};

use crate::config::BODY_ROT_SPEED;
use crate::entities::{Bullet, Tank, TrackMark};
use crate::math::{vec2_angle, vec2_length, vec2_sub};
use crate::world::World;

use super::combat::fire_bullet;
use super::modifiers::speed_multiplier;
use super::movement::{advance_tank, wrap_angle};

pub(super) fn update_player_tank(
    tank: &mut Tank,
    dt: f32,
    rl: &RaylibHandle,
    world: &World,
    mouse_world: Vector2,
    new_tracks: &mut Vec<TrackMark>,
    new_bullets: &mut Vec<Bullet>,
) {
    let mut turn: f32 = 0.0;
    if rl.is_key_down(KeyboardKey::KEY_A) || rl.is_key_down(KeyboardKey::KEY_LEFT) {
        turn -= 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) || rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        turn += 1.0;
    }
    if turn.abs() > 0.0 {
        tank.body_angle = wrap_angle(tank.body_angle + turn * BODY_ROT_SPEED * dt);
    }

    let mut movement: f32 = 0.0;
    if rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP) {
        movement += 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN) {
        movement -= 1.0;
    }

    if movement.abs() > 0.01 {
        advance_tank(tank, dt, movement * speed_multiplier(tank), world, new_tracks);
    }

    let turret_target = vec2_sub(mouse_world, tank.pos);
    if vec2_length(turret_target) > 0.01 {
        tank.turret_angle = vec2_angle(turret_target);
    }

    let wants_fire = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
        || rl.is_key_down(KeyboardKey::KEY_SPACE);
    if wants_fire && tank.fire_cooldown <= 0.0 {
        fire_bullet(tank, new_bullets);
    }
}
