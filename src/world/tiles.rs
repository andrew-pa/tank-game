use rand::{rngs::SmallRng, Rng};

use crate::config::TILE_SIZE;
use crate::entities::Team;

use super::SpawnZone;

#[derive(Clone, Copy, Debug)]
pub enum TileKind {
    Grass,
    Dirt,
    Sand,
}

pub(super) fn generate_tiles(rng: &mut SmallRng, width: i32, height: i32) -> Vec<TileKind> {
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

    tiles
}

pub(super) fn paint_spawn_zones(tiles: &mut [TileKind], width: i32, zones: &[SpawnZone; 2]) {
    for zone in zones {
        let start_x = (zone.rect.x / TILE_SIZE) as i32;
        let start_y = (zone.rect.y / TILE_SIZE) as i32;
        let end_x = ((zone.rect.x + zone.rect.width) / TILE_SIZE) as i32;
        let end_y = ((zone.rect.y + zone.rect.height) / TILE_SIZE) as i32;
        let tile_kind = match zone.team {
            Team::Red => TileKind::Grass,
            Team::Blue => TileKind::Sand,
        };

        for y in start_y..end_y {
            for x in start_x..end_x {
                let idx = (y * width + x) as usize;
                tiles[idx] = tile_kind;
            }
        }
    }
}
