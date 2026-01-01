use raylib::prelude::{Color, RaylibDraw};

use crate::assets::Assets;
use crate::math::vec2;

use super::helpers::{draw_tank_preview, draw_text_centered_screen, draw_texture_centered};
use super::Game;

impl Game {
    pub(super) fn draw_title<D: RaylibDraw>(
        &self,
        d: &mut D,
        assets: &Assets,
        screen_width: i32,
        screen_height: i32,
    ) {
        d.clear_background(Color::new(30, 85, 140, 255));
        let title = "TANKS: DOMINION";
        let title_size = 56;
        draw_text_centered_screen(
            d,
            title,
            40,
            title_size,
            Color::new(245, 245, 245, 255),
            screen_width,
        );

        let subtitle = "Two squads. Procedural frontier. Score the most eliminations.";
        let sub_size = 20;
        draw_text_centered_screen(
            d,
            subtitle,
            110,
            sub_size,
            Color::new(220, 220, 220, 255),
            screen_width,
        );

        let info = if self.input_state.gamepad_available() {
            "Press ENTER or START/A to deploy"
        } else {
            "Press ENTER to deploy"
        };
        let info_size = 28;
        draw_text_centered_screen(
            d,
            info,
            150,
            info_size,
            Color::new(240, 200, 110, 255),
            screen_width,
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
            if ox > screen_width as f32 - 90.0 {
                ox = 80.0;
                oy += 80.0;
            }
        }

        let mut smoke_x = screen_width as f32 - 220.0;
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
            if smoke_y > screen_height as f32 - 80.0 {
                smoke_y = 250.0;
                smoke_x += 80.0;
            }
        }
    }
}
