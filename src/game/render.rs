use raylib::prelude::*;

use crate::assets::{bullet_palette, obstacle_texture, tank_palette, Assets, TankPalette};
use crate::config::{TANK_RADIUS, TILE_SIZE, TRACK_LIFE, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::entities::{Explosion, Powerup, PowerupKind, SmokeColor, Tank, Team};
use crate::math::{rad_to_deg, vec2, vec2_angle, with_alpha};

use super::constants::SPRITE_ROT_OFFSET_DEG;
use super::{Game, ScreenState};

impl Game {
    pub fn draw(&self, d: &mut RaylibDrawHandle, assets: &Assets) {
        d.clear_background(Color::new(32, 96, 160, 255));
        match self.state {
            ScreenState::Title => self.draw_title(d, assets),
            ScreenState::Playing | ScreenState::RoundOver => {
                self.draw_world(d, assets);
                self.draw_hud(d);
                if self.state == ScreenState::RoundOver {
                    self.draw_round_over(d);
                }
            }
        }
    }

    fn draw_title(&self, d: &mut RaylibDrawHandle, assets: &Assets) {
        d.clear_background(Color::new(30, 85, 140, 255));
        let title = "TANKS: DOMINION";
        let title_size = 56;
        let title_width = d.measure_text(title, title_size);
        d.draw_text(
            title,
            (WINDOW_WIDTH - title_width) / 2,
            40,
            title_size,
            Color::new(245, 245, 245, 255),
        );

        let subtitle = "Two squads. Procedural frontier. Score the most eliminations.";
        let sub_size = 20;
        let sub_width = d.measure_text(subtitle, sub_size);
        d.draw_text(
            subtitle,
            (WINDOW_WIDTH - sub_width) / 2,
            110,
            sub_size,
            Color::new(220, 220, 220, 255),
        );

        let info = "Press ENTER to deploy";
        let info_size = 28;
        let info_width = d.measure_text(info, info_size);
        d.draw_text(
            info,
            (WINDOW_WIDTH - info_width) / 2,
            150,
            info_size,
            Color::new(240, 200, 110, 255),
        );

        let mut x = 120.0;
        let y = 240.0;
        let tank_line = [
            &assets.tanks.red,
            &assets.tanks.blue,
            &assets.tanks.green,
            &assets.tanks.beige,
            &assets.tanks.black,
        ];
        for palette in tank_line {
            draw_tank_preview(d, palette, vec2(x, y));
            x += 140.0;
        }

        let bullet_y = 350.0;
        let bullets = [
            &assets.bullets.red,
            &assets.bullets.blue,
            &assets.bullets.green,
            &assets.bullets.beige,
            &assets.bullets.yellow,
            &assets.bullets.silver,
        ];
        let mut bx = 120.0;
        for bullet in bullets {
            draw_texture_centered(d, &bullet.normal, vec2(bx, bullet_y), 0.0, Color::WHITE);
            draw_texture_centered(
                d,
                &bullet.silver,
                vec2(bx, bullet_y + 40.0),
                0.0,
                Color::WHITE,
            );
            draw_texture_centered(
                d,
                &bullet.outline,
                vec2(bx, bullet_y + 80.0),
                0.0,
                Color::WHITE,
            );
            draw_texture_centered(
                d,
                &bullet.silver_outline,
                vec2(bx, bullet_y + 120.0),
                0.0,
                Color::WHITE,
            );
            bx += 100.0;
        }

        let obstacles = [
            &assets.obstacles.tree_small,
            &assets.obstacles.tree_large,
            &assets.obstacles.sandbag_brown,
            &assets.obstacles.sandbag_beige,
            &assets.obstacles.oil,
            &assets.obstacles.barrel_red_up,
            &assets.obstacles.barrel_red_side,
            &assets.obstacles.barrel_grey_up,
            &assets.obstacles.barrel_grey_side,
            &assets.obstacles.barrel_grey_rust,
            &assets.obstacles.barrel_green_up,
            &assets.obstacles.barrel_green_side,
            &assets.obstacles.barrel_green_side_damaged,
        ];
        let mut ox = 80.0;
        let mut oy = 520.0;
        for texture in obstacles {
            draw_texture_centered(d, texture, vec2(ox, oy), 0.0, Color::WHITE);
            ox += 90.0;
            if ox > WINDOW_WIDTH as f32 - 90.0 {
                ox = 80.0;
                oy += 80.0;
            }
        }

        let mut smoke_x = WINDOW_WIDTH as f32 - 220.0;
        let mut smoke_y = 250.0;
        let smoke_frames = assets
            .smoke
            .orange
            .iter()
            .chain(assets.smoke.yellow.iter())
            .chain(assets.smoke.grey.iter())
            .chain(assets.smoke.white.iter());
        for frame in smoke_frames {
            draw_texture_centered(d, frame, vec2(smoke_x, smoke_y), 0.0, Color::WHITE);
            smoke_y += 50.0;
            if smoke_y > WINDOW_HEIGHT as f32 - 80.0 {
                smoke_y = 250.0;
                smoke_x += 80.0;
            }
        }
    }

    fn draw_world(&self, d: &mut RaylibDrawHandle, assets: &Assets) {
        let camera = self.camera();
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
                if let Some(player) = self
                    .tanks
                    .get(self.player_index)
                    .filter(|tank| tank.alive)
                {
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
                    let width = d2.measure_text(label, size);
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

    fn draw_hud(&self, d: &mut RaylibDrawHandle) {
        let bar_height = 48;
        d.draw_rectangle(0, 0, WINDOW_WIDTH, bar_height, Color::new(20, 24, 28, 220));
        let red_label = format!("{}: {}", Team::Red.name(), self.team_kills[0]);
        let blue_label = format!("{}: {}", Team::Blue.name(), self.team_kills[1]);
        d.draw_text(&red_label, 20, 12, 20, Team::Red.color());
        let blue_width = d.measure_text(&blue_label, 20);
        d.draw_text(
            &blue_label,
            WINDOW_WIDTH - blue_width - 20,
            12,
            20,
            Team::Blue.color(),
        );
        let time_label = format!("Time: {:>3.0}", self.round_timer.ceil());
        let time_width = d.measure_text(&time_label, 20);
        d.draw_text(
            &time_label,
            (WINDOW_WIDTH - time_width) / 2,
            12,
            20,
            Color::new(240, 240, 240, 255),
        );

        if self.countdown_timer > 0.0 {
            self.draw_countdown(d);
        }

        if let Some(player) = self.tanks.get(self.player_index) {
            self.draw_player_health(d, player);
            if !player.alive {
                self.draw_respawn_notice(d, player.respawn_timer);
            }
        }
    }

    fn draw_round_over(&self, d: &mut RaylibDrawHandle) {
        d.draw_rectangle(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT, Color::new(10, 10, 10, 160));
        let message = match self.last_winner {
            Some(team) => format!("{} wins the round!", team.name()),
            None => "Stalemate!".to_string(),
        };
        let size = 46;
        let width = d.measure_text(&message, size);
        d.draw_text(
            &message,
            (WINDOW_WIDTH - width) / 2,
            WINDOW_HEIGHT / 2 - 40,
            size,
            Color::new(240, 240, 240, 255),
        );
        let prompt = "Press ENTER to redeploy";
        let prompt_size = 24;
        let prompt_width = d.measure_text(prompt, prompt_size);
        d.draw_text(
            prompt,
            (WINDOW_WIDTH - prompt_width) / 2,
            WINDOW_HEIGHT / 2 + 20,
            prompt_size,
            Color::new(220, 200, 120, 255),
        );
    }

    fn draw_respawn_notice(&self, d: &mut RaylibDrawHandle, timer: f32) {
        let remaining = timer.ceil().max(1.0) as i32;
        d.draw_rectangle(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT, Color::new(8, 8, 12, 150));
        let text = format!("Eliminated! Respawning in {remaining}");
        let size = 38;
        let width = d.measure_text(&text, size);
        d.draw_text(
            &text,
            (WINDOW_WIDTH - width) / 2,
            WINDOW_HEIGHT / 2 - 20,
            size,
            Color::new(240, 210, 120, 240),
        );
    }

    fn draw_player_health(&self, d: &mut RaylibDrawHandle, player: &Tank) {
        let bar_width = 260;
        let bar_height = 14;
        let x = 20;
        let y = 58;
        let pct = (player.health / player.max_health).clamp(0.0, 1.0);
        d.draw_rectangle(x, y, bar_width, bar_height, Color::new(10, 10, 10, 200));
        d.draw_rectangle(
            x + 2,
            y + 2,
            ((bar_width - 4) as f32 * pct) as i32,
            bar_height - 4,
            if player.invincible_timer > 0.0 {
                Color::new(120, 210, 255, 230)
            } else {
                player.team.color()
            },
        );
        d.draw_text("Hull", x, y - 18, 16, Color::new(230, 230, 230, 220));
    }

    fn draw_countdown(&self, d: &mut RaylibDrawHandle) {
        let count = self.countdown_timer.ceil().max(1.0) as i32;
        let text = format!("Deploying in {count}");
        let size = 44;
        let width = d.measure_text(&text, size);
        d.draw_text(
            &text,
            (WINDOW_WIDTH - width) / 2,
            WINDOW_HEIGHT / 2 - 80,
            size,
            Color::new(255, 230, 120, 240),
        );
        let hint = "WASD to move • Mouse to aim • LMB to fire";
        let hint_size = 18;
        let hint_width = d.measure_text(hint, hint_size);
        d.draw_text(
            hint,
            (WINDOW_WIDTH - hint_width) / 2,
            WINDOW_HEIGHT / 2 - 32,
            hint_size,
            Color::new(230, 230, 230, 220),
        );
    }
}

fn explosion_frame<'a>(assets: &'a Assets, explosion: &Explosion) -> &'a Texture2D {
    let frame = (explosion.age / 0.08).floor() as usize;
    let idx = frame.min(5);
    match explosion.color {
        SmokeColor::Orange => &assets.smoke.orange[idx],
        SmokeColor::Yellow => &assets.smoke.yellow[idx],
        SmokeColor::Grey => &assets.smoke.grey[idx],
        SmokeColor::White => &assets.smoke.white[idx],
    }
}

fn draw_tank_preview(d: &mut RaylibDrawHandle, palette: &TankPalette, pos: Vector2) {
    draw_texture_centered(
        d,
        &palette.outline_body,
        vec2(pos.x + 2.0, pos.y + 2.0),
        sprite_rotation(0.0),
        Color::new(0, 0, 0, 90),
    );
    draw_texture_centered(d, &palette.body, pos, sprite_rotation(0.0), Color::WHITE);
    draw_barrel(
        d,
        &palette.outline_barrel,
        pos,
        0.3,
        Color::new(0, 0, 0, 90),
    );
    draw_barrel(d, &palette.barrel, pos, 0.3, Color::WHITE);
}

fn draw_texture_centered(
    d: &mut RaylibDrawHandle,
    texture: &Texture2D,
    pos: Vector2,
    rotation: f32,
    tint: Color,
) {
    let w = texture.width as f32;
    let h = texture.height as f32;
    let dest = Rectangle {
        x: pos.x,
        y: pos.y,
        width: w,
        height: h,
    };
    let src = Rectangle {
        x: 0.0,
        y: 0.0,
        width: w,
        height: h,
    };
    d.draw_texture_pro(
        texture,
        src,
        dest,
        Vector2 { x: w / 2.0, y: h / 2.0 },
        rotation,
        tint,
    );
}

fn draw_barrel(
    d: &mut RaylibDrawHandle,
    texture: &Texture2D,
    pos: Vector2,
    angle: f32,
    tint: Color,
) {
    let w = texture.width as f32;
    let h = texture.height as f32;
    let origin = Vector2 { x: w / 2.0, y: h };
    let dest = Rectangle {
        x: pos.x,
        y: pos.y,
        width: w,
        height: h,
    };
    let src = Rectangle {
        x: 0.0,
        y: 0.0,
        width: w,
        height: h,
    };
    d.draw_texture_pro(
        texture,
        src,
        dest,
        origin,
        sprite_rotation(angle),
        tint,
    );
}

fn sprite_rotation(angle: f32) -> f32 {
    rad_to_deg(angle) + SPRITE_ROT_OFFSET_DEG
}

fn draw_tank_health(d: &mut RaylibDrawHandle, tank: &Tank) {
    if tank.health >= tank.max_health || tank.health_flash <= 0.0 {
        return;
    }
    let pct = (tank.health / tank.max_health).clamp(0.0, 1.0);
    let bar_w = 44.0;
    let bar_h = 6.0;
    let x = tank.pos.x - bar_w * 0.5;
    let y = tank.pos.y - TANK_RADIUS - 16.0;
    d.draw_rectangle(x as i32, y as i32, bar_w as i32, bar_h as i32, Color::new(10, 10, 10, 190));
    d.draw_rectangle(
        (x + 1.0) as i32,
        (y + 1.0) as i32,
        ((bar_w - 2.0) * pct) as i32,
        (bar_h - 2.0) as i32,
        tank.team.color(),
    );
}

fn draw_powerup_markers(d: &mut RaylibDrawHandle, tank: &Tank) {
    let mut ring = 0.0;
    if tank.invincible_timer > 0.0 {
        ring += 1.0;
        let pulse = (tank.invincible_timer * 4.0).sin().abs();
        d.draw_circle_lines(
            tank.pos.x as i32,
            tank.pos.y as i32,
            TANK_RADIUS + 8.0 + pulse * 4.0,
            Color::new(120, 210, 255, 220),
        );
    }
    if tank.rapid_timer > 0.0 {
        ring += 1.0;
        let pulse = (tank.rapid_timer * 5.0).sin().abs();
        d.draw_circle_lines(
            tank.pos.x as i32,
            tank.pos.y as i32,
            TANK_RADIUS + 4.0 + ring * 6.0 + pulse * 3.0,
            Color::new(255, 200, 90, 220),
        );
    }
}

fn draw_powerup(d: &mut RaylibDrawHandle, assets: &Assets, powerup: &Powerup) {
    let frames = match powerup.kind {
        PowerupKind::Invincible => &assets.smoke.white,
        PowerupKind::RapidRange => &assets.smoke.orange,
        PowerupKind::Heal => &assets.smoke.yellow,
    };
    let idx = ((powerup.age * 8.0) as usize) % frames.len();
    let pulse = (powerup.age * 6.0).sin().abs();
    let tint = match powerup.kind {
        PowerupKind::Invincible => Color::new(120, 210, 255, 235),
        PowerupKind::RapidRange => Color::new(255, 200, 90, 235),
        PowerupKind::Heal => Color::new(120, 230, 160, 235),
    };
    draw_texture_centered(d, &frames[idx], powerup.pos, 0.0, tint);
    d.draw_circle_lines(
        powerup.pos.x as i32,
        powerup.pos.y as i32,
        26.0 + pulse * 6.0,
        match powerup.kind {
            PowerupKind::Invincible => Color::new(120, 210, 255, 220),
            PowerupKind::RapidRange => Color::new(255, 200, 90, 220),
            PowerupKind::Heal => Color::new(120, 230, 160, 220),
        },
    );
}
