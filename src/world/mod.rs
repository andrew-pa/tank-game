mod generation;
mod obstacles;
mod tiles;

use rand::{Rng, rngs::SmallRng};
use raylib::prelude::{Rectangle, Vector2};

use crate::config::TILE_SIZE;
use crate::entities::Team;
use crate::math::vec2;

pub use obstacles::{Obstacle, ObstacleKind};
pub use tiles::TileKind;

#[derive(Clone, Copy, Debug)]
pub struct SpawnZone {
    pub rect: Rectangle,
    pub team: Team,
}

impl SpawnZone {
    pub fn contains(&self, pos: Vector2) -> bool {
        pos.x >= self.rect.x
            && pos.x <= self.rect.x + self.rect.width
            && pos.y >= self.rect.y
            && pos.y <= self.rect.y + self.rect.height
    }
}

pub struct World {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<TileKind>,
    pub obstacles: Vec<Obstacle>,
    pub spawn_zones: [SpawnZone; 2],
}

impl World {
    pub fn new(rng: &mut SmallRng) -> Self {
        generation::generate_world(rng)
    }

    pub fn index(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn tile_kind(&self, x: i32, y: i32) -> TileKind {
        self.tiles[self.index(x, y)]
    }

    pub fn world_bounds(&self) -> Rectangle {
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: self.width as f32 * TILE_SIZE,
            height: self.height as f32 * TILE_SIZE,
        }
    }

    pub fn random_point_in_zone(&self, team: Team, rng: &mut SmallRng) -> Vector2 {
        let zone = self
            .spawn_zones
            .iter()
            .find(|zone| zone.team == team)
            .expect("missing spawn zone");
        let margin = TILE_SIZE * 0.4;
        let x = rng.random_range(zone.rect.x + margin..zone.rect.x + zone.rect.width - margin);
        let y = rng.random_range(zone.rect.y + margin..zone.rect.y + zone.rect.height - margin);
        vec2(x, y)
    }

    pub fn is_inside_spawn_zone(&self, pos: Vector2) -> bool {
        self.spawn_zones.iter().any(|zone| zone.contains(pos))
    }

    pub fn is_inside_enemy_zone(&self, team: Team, pos: Vector2) -> bool {
        self.spawn_zones
            .iter()
            .any(|zone| zone.team != team && zone.contains(pos))
    }
}
