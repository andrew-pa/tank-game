use raylib::prelude::Vector2;

use crate::config::BODY_ROT_SPEED;
use crate::entities::{Bullet, Tank, TrackMark};
use crate::math::{vec2_angle, vec2_length, vec2_sub};
use crate::world::World;

use super::super::input::PlayerInput;
use super::combat::fire_bullet;
use super::modifiers::speed_multiplier;
use super::movement::{advance_tank, wrap_angle};

pub(super) fn update_player_tank(
    tank: &mut Tank,
    dt: f32,
    world: &World,
    mouse_world: Vector2,
    input: &PlayerInput,
    new_tracks: &mut Vec<TrackMark>,
    new_bullets: &mut Vec<Bullet>,
) {
    if input.turn.abs() > 0.0 {
        tank.body_angle = wrap_angle(tank.body_angle + input.turn * BODY_ROT_SPEED * dt);
    }

    if input.movement.abs() > 0.01 {
        advance_tank(
            tank,
            dt,
            input.movement * speed_multiplier(tank),
            world,
            new_tracks,
        );
    }

    if let Some(aim_dir) = input.aim_dir {
        if vec2_length(aim_dir) > 0.01 {
            tank.turret_angle = vec2_angle(aim_dir);
        }
    } else if input.use_mouse_aim {
        let turret_target = vec2_sub(mouse_world, tank.pos);
        if vec2_length(turret_target) > 0.01 {
            tank.turret_angle = vec2_angle(turret_target);
        }
    }

    if input.wants_fire && tank.fire_cooldown <= 0.0 {
        fire_bullet(tank, new_bullets);
    }
}
