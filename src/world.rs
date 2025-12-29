use rand::{rngs::SmallRng, Rng};
use raylib::prelude::{Rectangle, Vector2};

use crate::config::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE};
use crate::entities::Team;
use crate::math::{vec2, vec2_distance};

#[derive(Clone, Copy, Debug)]
pub enum TileKind {
    Grass,
    Dirt,
    Sand,
}

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

pub struct World {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<TileKind>,
    pub obstacles: Vec<Obstacle>,
    pub spawn_zones: [SpawnZone; 2],
}

impl World {
    pub fn new(rng: &mut SmallRng) -> Self {
        let width = MAP_WIDTH;
        let height = MAP_HEIGHT;
        let mut values = vec![0.0f32; (width * height) as usize];
        for value in &mut values {
            *value = rng.random_range(0.0..1.0);
        }

        for _ in 0..3 {
            let mut next = values.clone();
            for y in 0..height {
                for x in 0..width {
                    let mut sum = 0.0;
                    let mut count = 0.0;
                    for ny in (y - 1).max(0)..=(y + 1).min(height - 1) {
                        for nx in (x - 1).max(0)..=(x + 1).min(width - 1) {
                            let idx = (ny * width + nx) as usize;
                            sum += values[idx];
                            count += 1.0;
                        }
                    }
                    let idx = (y * width + x) as usize;
                    next[idx] = sum / count;
                }
            }
            values = next;
        }

        let mut tiles = vec![TileKind::Grass; (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                let v = values[idx];
                tiles[idx] = if v < 0.35 {
                    TileKind::Sand
                } else if v < 0.62 {
                    TileKind::Dirt
                } else {
                    TileKind::Grass
                };
            }
        }

        let zone_w = 7;
        let zone_h = 8;
        let zone_y = (height - zone_h) / 2;
        let left_zone = SpawnZone {
            rect: Rectangle {
                x: 2.0 * TILE_SIZE,
                y: zone_y as f32 * TILE_SIZE,
                width: zone_w as f32 * TILE_SIZE,
                height: zone_h as f32 * TILE_SIZE,
            },
            team: Team::Red,
        };
        let right_zone = SpawnZone {
            rect: Rectangle {
                x: (width - zone_w - 2) as f32 * TILE_SIZE,
                y: zone_y as f32 * TILE_SIZE,
                width: zone_w as f32 * TILE_SIZE,
                height: zone_h as f32 * TILE_SIZE,
            },
            team: Team::Blue,
        };

        for y in zone_y..zone_y + zone_h {
            for x in 2..2 + zone_w {
                let idx = (y * width + x) as usize;
                tiles[idx] = TileKind::Grass;
            }
            for x in width - zone_w - 2..width - 2 {
                let idx = (y * width + x) as usize;
                tiles[idx] = TileKind::Sand;
            }
        }

        let mut world = Self {
            width,
            height,
            tiles,
            obstacles: Vec::new(),
            spawn_zones: [left_zone, right_zone],
        };
        world.generate_obstacles(rng);
        world
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

    pub fn is_inside_enemy_zone(&self, team: Team, pos: Vector2) -> bool {
        self.spawn_zones
            .iter()
            .any(|zone| zone.team != team && zone.contains(pos))
    }

    fn generate_obstacles(&mut self, rng: &mut SmallRng) {
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
            if let Some(pos) = self.find_open_obstacle_position(kind, rng, 120) {
                self.obstacles.push(Obstacle {
                    kind,
                    pos,
                    radius: obstacle_radius(kind),
                });
            }
        }

        let target = ((self.width * self.height) as f32 * 0.08) as usize;
        let mut attempts = 0;
        while self.obstacles.len() < target && attempts < target * 8 {
            attempts += 1;
            let kind = all_kinds[rng.random_range(0..all_kinds.len())];
            if let Some(pos) = self.find_open_obstacle_position(kind, rng, 40) {
                self.obstacles.push(Obstacle {
                    kind,
                    pos,
                    radius: obstacle_radius(kind),
                });
            }
        }
    }

    fn find_open_obstacle_position(
        &self,
        kind: ObstacleKind,
        rng: &mut SmallRng,
        attempts: usize,
    ) -> Option<Vector2> {
        let bounds = self.world_bounds();
        let radius = obstacle_radius(kind);
        for _ in 0..attempts {
            let x = rng.random_range(bounds.x + radius..bounds.x + bounds.width - radius);
            let y = rng.random_range(bounds.y + radius..bounds.y + bounds.height - radius);
            let pos = vec2(x, y);
            if self.is_inside_enemy_zone(Team::Red, pos) || self.is_inside_enemy_zone(Team::Blue, pos) {
                continue;
            }
            if self
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
}

pub fn obstacle_radius(kind: ObstacleKind) -> f32 {
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
