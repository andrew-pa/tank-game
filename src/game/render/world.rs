use raylib::prelude::{Color, RaylibDraw, RaylibMode2DExt};

use crate::assets::{Assets, bullet_palette, obstacle_texture, tank_palette};
use crate::config::{TANK_RADIUS, TILE_SIZE, TRACK_LIFE};
use crate::math::{vec2, vec2_angle, with_alpha};

use super::Game;
use super::helpers::{
    draw_barrel, draw_powerup, draw_powerup_markers, draw_tank_health, draw_texture_centered,
    explosion_frame, measure_text_width, sprite_rotation,
};

impl Game {
    pub(super) fn draw_world<D: RaylibDraw>(
        &self,
        d: &mut D,
        assets: &Assets,
        screen_width: i32,
        screen_height: i32,
    ) {
        let camera = self.camera(screen_width, screen_height);
        d.draw_mode2D(camera, |mut d2, _| {
            for y in 0..self.world.height {
                for x in 0..self.world.width {
                    let texture = match self.world.tile_kind(x, y) {
                        crate::world::TileKind::Grass => &assets.tiles.grass,
                        crate::world::TileKind::Dirt => &assets.tiles.dirt,
                        crate::world::TileKind::Sand => &assets.tiles.sand,
                    };
                    let pos = vec2(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
                    d2.draw_texture_ex(texture, pos, 0.0, 1.0, Color::WHITE);
                }
            }

            for zone in &self.world.spawn_zones {
                let tint = with_alpha(zone.team.color(), 0.18);
                d2.draw_rectangle_rec(zone.rect, tint);
            }

            for track in &self.tracks {
                let alpha = (1.0 - track.age / TRACK_LIFE).max(0.0);
                let tint = with_alpha(Color::new(200, 200, 200, 255), alpha * 0.7);
                let texture = if (track.age * 10.0) as i32 % 2 == 0 {
                    &assets.tracks_large
                } else {
                    &assets.tracks_small
                };
                draw_texture_centered(
                    &mut d2,
                    texture,
                    track.pos,
                    sprite_rotation(track.rotation),
                    tint,
                );
            }

            for obstacle in &self.world.obstacles {
                let texture = obstacle_texture(assets, obstacle.kind);
                draw_texture_centered(&mut d2, texture, obstacle.pos, 0.0, Color::WHITE);
            }

            for powerup in &self.powerups {
                draw_powerup(&mut d2, assets, powerup);
            }

            for tank in &self.tanks {
                if !tank.alive {
                    continue;
                }
                let palette = tank_palette(assets, tank.team);
                let tread_texture = if (tank.tread_phase as i32) % 2 == 0 {
                    &assets.tracks_large
                } else {
                    &assets.tracks_small
                };
                draw_texture_centered(
                    &mut d2,
                    tread_texture,
                    tank.pos,
                    sprite_rotation(tank.body_angle),
                    with_alpha(Color::new(160, 160, 160, 255), 0.35),
                );

                draw_texture_centered(
                    &mut d2,
                    &palette.outline_body,
                    vec2(tank.pos.x + 2.0, tank.pos.y + 2.0),
                    sprite_rotation(tank.body_angle),
                    Color::new(0, 0, 0, 90),
                );
                draw_texture_centered(
                    &mut d2,
                    &palette.body,
                    tank.pos,
                    sprite_rotation(tank.body_angle),
                    Color::WHITE,
                );
                draw_barrel(
                    &mut d2,
                    &palette.outline_barrel,
                    tank.pos,
                    tank.turret_angle,
                    Color::new(0, 0, 0, 90),
                );
                draw_barrel(
                    &mut d2,
                    &palette.barrel,
                    tank.pos,
                    tank.turret_angle,
                    Color::WHITE,
                );

                draw_tank_health(&mut d2, tank);
                draw_powerup_markers(&mut d2, tank);
            }

            for bullet in &self.bullets {
                let palette = bullet_palette(assets, bullet.team);
                let rotation = sprite_rotation(vec2_angle(bullet.vel));
                draw_texture_centered(&mut d2, &palette.normal, bullet.pos, rotation, Color::WHITE);
            }

            for explosion in &self.explosions {
                let frame = explosion_frame(assets, explosion);
                draw_texture_centered(&mut d2, frame, explosion.pos, 0.0, Color::WHITE);
            }

            if self.intro_timer > 0.0 {
                if let Some(player) = self.tanks.get(self.player_index).filter(|tank| tank.alive) {
                    let pulse = (self.intro_timer * 6.0).sin().abs();
                    let radius = TANK_RADIUS + 10.0 + pulse * 6.0;
                    d2.draw_circle_lines(
                        player.pos.x as i32,
                        player.pos.y as i32,
                        radius,
                        Color::new(255, 230, 120, 220),
                    );
                    let label = "YOU";
                    let size = 18;
                    let width = measure_text_width(label, size);
                    d2.draw_text(
                        label,
                        (player.pos.x - width as f32 * 0.5) as i32,
                        (player.pos.y - radius - 24.0) as i32,
                        size,
                        Color::new(255, 230, 120, 235),
                    );
                }
            }
        });
    }
}
