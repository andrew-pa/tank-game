use rand::rngs::SmallRng;
use raylib::prelude::Rectangle;

use crate::config::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE};
use crate::entities::Team;

use super::obstacles;
use super::tiles;
use super::{SpawnZone, World};

pub(super) fn generate_world(rng: &mut SmallRng) -> World {
    let width = MAP_WIDTH;
    let height = MAP_HEIGHT;

    let mut tiles = tiles::generate_tiles(rng, width, height);
    let spawn_zones = spawn_zones(width, height);
    tiles::paint_spawn_zones(&mut tiles, width, &spawn_zones);

    let mut world = World {
        width,
        height,
        tiles,
        obstacles: Vec::new(),
        spawn_zones,
    };
    obstacles::generate_obstacles(&mut world, rng);
    world
}

fn spawn_zones(width: i32, height: i32) -> [SpawnZone; 2] {
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

    [left_zone, right_zone]
}
