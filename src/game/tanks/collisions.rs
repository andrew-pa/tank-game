use crate::config::TANK_RADIUS;
use crate::entities::Tank;
use crate::math::{push_outside_rect, vec2_add, vec2_length, vec2_normalize, vec2_scale, vec2_sub};
use crate::world::World;

pub(super) fn resolve_tank_collisions(tanks: &mut [Tank], world: &World) {
    for i in 0..tanks.len() {
        for j in (i + 1)..tanks.len() {
            if !tanks[i].alive || !tanks[j].alive {
                continue;
            }
            let delta = vec2_sub(tanks[j].pos, tanks[i].pos);
            let dist = vec2_length(delta);
            let min_dist = TANK_RADIUS * 2.0 - 2.0;
            if dist > 0.0 && dist < min_dist {
                let push = vec2_scale(vec2_normalize(delta), (min_dist - dist) * 0.5);
                tanks[i].pos = vec2_sub(tanks[i].pos, push);
                tanks[j].pos = vec2_add(tanks[j].pos, push);
            }
        }
    }

    for tank in tanks {
        if !tank.alive {
            continue;
        }
        if let Some(zone) = world
            .spawn_zones
            .iter()
            .find(|zone| zone.team == tank.team.enemy())
        {
            tank.pos = push_outside_rect(tank.pos, zone.rect, TANK_RADIUS + 2.0);
        }
    }
}
