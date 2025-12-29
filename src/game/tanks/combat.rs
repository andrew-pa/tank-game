use crate::config::{BULLET_LIFE, BULLET_SPEED, FIRE_COOLDOWN};
use crate::entities::{Bullet, Tank};
use crate::math::{vec2_add, vec2_from_angle, vec2_scale};

use super::super::constants::BARREL_LENGTH;
use super::modifiers::{fire_rate_multiplier, range_multiplier};

pub(super) fn fire_bullet(tank: &mut Tank, new_bullets: &mut Vec<Bullet>) {
    new_bullets.push(bullet_from_tank(tank));
    tank.fire_cooldown = FIRE_COOLDOWN / fire_rate_multiplier(tank);
}

fn bullet_from_tank(tank: &Tank) -> Bullet {
    let dir = vec2_from_angle(tank.turret_angle);
    Bullet {
        pos: vec2_add(tank.pos, vec2_scale(dir, BARREL_LENGTH)),
        vel: vec2_scale(dir, BULLET_SPEED),
        team: tank.team,
        life: BULLET_LIFE * range_multiplier(tank),
    }
}
