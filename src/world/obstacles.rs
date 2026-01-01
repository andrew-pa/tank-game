use rand::{Rng, rngs::SmallRng};
use raylib::prelude::Vector2;

use crate::math::{vec2, vec2_distance};

use super::World;

#[derive(Clone, Copy, Debug)]
pub enum ObstacleKind {
    TreeSmall,
    TreeLarge,
    SandbagBrown,
    SandbagBeige,
    Oil,
    BarrelRedUp,
    BarrelRedSide,
    BarrelGreyUp,
    BarrelGreySide,
    BarrelGreyRust,
    BarrelGreenUp,
    BarrelGreenSide,
    BarrelGreenSideDamaged,
}

#[derive(Clone, Debug)]
pub struct Obstacle {
    pub kind: ObstacleKind,
    pub pos: Vector2,
    pub radius: f32,
}

pub(super) fn generate_obstacles(world: &mut World, rng: &mut SmallRng) {
    let all_kinds = [
        ObstacleKind::TreeSmall,
        ObstacleKind::TreeLarge,
        ObstacleKind::SandbagBrown,
        ObstacleKind::SandbagBeige,
        ObstacleKind::Oil,
        ObstacleKind::BarrelRedUp,
        ObstacleKind::BarrelRedSide,
        ObstacleKind::BarrelGreyUp,
        ObstacleKind::BarrelGreySide,
        ObstacleKind::BarrelGreyRust,
        ObstacleKind::BarrelGreenUp,
        ObstacleKind::BarrelGreenSide,
        ObstacleKind::BarrelGreenSideDamaged,
    ];

    for kind in all_kinds {
        if let Some(pos) = find_open_obstacle_position(world, kind, rng, 120) {
            world.obstacles.push(Obstacle {
                kind,
                pos,
                radius: obstacle_radius(kind),
            });
        }
    }

    let target = ((world.width * world.height) as f32 * 0.08) as usize;
    let mut attempts = 0;
    while world.obstacles.len() < target && attempts < target * 8 {
        attempts += 1;
        let kind = all_kinds[rng.random_range(0..all_kinds.len())];
        if let Some(pos) = find_open_obstacle_position(world, kind, rng, 40) {
            world.obstacles.push(Obstacle {
                kind,
                pos,
                radius: obstacle_radius(kind),
            });
        }
    }
}

fn find_open_obstacle_position(
    world: &World,
    kind: ObstacleKind,
    rng: &mut SmallRng,
    attempts: usize,
) -> Option<Vector2> {
    let bounds = world.world_bounds();
    let radius = obstacle_radius(kind);
    for _ in 0..attempts {
        let x = rng.random_range(bounds.x + radius..bounds.x + bounds.width - radius);
        let y = rng.random_range(bounds.y + radius..bounds.y + bounds.height - radius);
        let pos = vec2(x, y);
        if world.is_inside_spawn_zone(pos) {
            continue;
        }
        if world
            .obstacles
            .iter()
            .any(|obs| vec2_distance(obs.pos, pos) < obs.radius + radius + 16.0)
        {
            continue;
        }
        return Some(pos);
    }
    None
}

fn obstacle_radius(kind: ObstacleKind) -> f32 {
    match kind {
        ObstacleKind::TreeSmall => 40.0,
        ObstacleKind::TreeLarge => 60.0,
        ObstacleKind::SandbagBrown | ObstacleKind::SandbagBeige => 30.0,
        ObstacleKind::Oil => 24.0,
        ObstacleKind::BarrelRedUp
        | ObstacleKind::BarrelRedSide
        | ObstacleKind::BarrelGreyUp
        | ObstacleKind::BarrelGreySide
        | ObstacleKind::BarrelGreyRust
        | ObstacleKind::BarrelGreenUp
        | ObstacleKind::BarrelGreenSide
        | ObstacleKind::BarrelGreenSideDamaged => 22.0,
    }
}
