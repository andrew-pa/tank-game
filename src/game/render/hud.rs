use raylib::prelude::{Color, RaylibDraw};

use crate::entities::{Tank, Team};
use crate::game::input::InputDevice;

use super::Game;
use super::helpers::{draw_text_centered_screen, measure_text_width};

impl Game {
    pub(super) fn draw_hud<D: RaylibDraw>(&self, d: &mut D, screen_width: i32, screen_height: i32) {
        let bar_height = 48;
        d.draw_rectangle(0, 0, screen_width, bar_height, Color::new(20, 24, 28, 220));
        let red_label = format!("{}: {}", Team::Red.name(), self.team_kills[0]);
        let blue_label = format!("{}: {}", Team::Blue.name(), self.team_kills[1]);
        d.draw_text(&red_label, 20, 12, 20, Team::Red.color());
        let blue_width = measure_text_width(&blue_label, 20);
        d.draw_text(
            &blue_label,
            screen_width - blue_width - 20,
            12,
            20,
            Team::Blue.color(),
        );
        let time_label = format!("Time: {:>3.0}", self.round_timer.ceil());
        let time_width = measure_text_width(&time_label, 20);
        d.draw_text(
            &time_label,
            (screen_width - time_width) / 2,
            12,
            20,
            Color::new(240, 240, 240, 255),
        );

        if self.countdown_timer > 0.0 {
            self.draw_countdown(d, screen_width, screen_height);
        }

        if let Some(player) = self.tanks.get(self.player_index) {
            self.draw_player_health(d, player);
            if !player.alive {
                self.draw_respawn_notice(d, player.respawn_timer, screen_width, screen_height);
            }
        }
    }

    pub(super) fn draw_round_over<D: RaylibDraw>(
        &self,
        d: &mut D,
        screen_width: i32,
        screen_height: i32,
    ) {
        d.draw_rectangle(
            0,
            0,
            screen_width,
            screen_height,
            Color::new(10, 10, 10, 160),
        );
        let message = match self.last_winner {
            Some(team) => format!("{} wins the round!", team.name()),
            None => "Stalemate!".to_string(),
        };
        let size = 46;
        draw_text_centered_screen(
            d,
            &message,
            screen_height / 2 - 40,
            size,
            Color::new(240, 240, 240, 255),
            screen_width,
        );
        let prompt = if self.input_state.gamepad_available() {
            "Press ENTER or START/A to redeploy"
        } else {
            "Press ENTER to redeploy"
        };
        let prompt_size = 24;
        draw_text_centered_screen(
            d,
            prompt,
            screen_height / 2 + 20,
            prompt_size,
            Color::new(220, 200, 120, 255),
            screen_width,
        );
    }

    fn draw_respawn_notice<D: RaylibDraw>(
        &self,
        d: &mut D,
        timer: f32,
        screen_width: i32,
        screen_height: i32,
    ) {
        let remaining = timer.ceil().max(1.0) as i32;
        d.draw_rectangle(0, 0, screen_width, screen_height, Color::new(8, 8, 12, 150));
        let text = format!("Eliminated! Respawning in {remaining}");
        let size = 38;
        draw_text_centered_screen(
            d,
            &text,
            screen_height / 2 - 20,
            size,
            Color::new(240, 210, 120, 240),
            screen_width,
        );
    }

    fn draw_player_health<D: RaylibDraw>(&self, d: &mut D, player: &Tank) {
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

    fn draw_countdown<D: RaylibDraw>(&self, d: &mut D, screen_width: i32, screen_height: i32) {
        let count = self.countdown_timer.ceil().max(1.0) as i32;
        let text = format!("Deploying in {count}");
        let size = 44;
        draw_text_centered_screen(
            d,
            &text,
            screen_height / 2 - 80,
            size,
            Color::new(255, 230, 120, 240),
            screen_width,
        );
        let hint = if self.input_state.gamepad_available()
            && self.input_state.last_device() == InputDevice::Gamepad
        {
            "Left Stick to move • Right Stick to aim • RT to fire"
        } else if self.input_state.gamepad_available() {
            "WASD/Arrows or Left Stick to move • Mouse or Right Stick to aim • LMB/Space or RT to fire"
        } else {
            "WASD/Arrows to move • Mouse to aim • LMB/Space to fire"
        };
        let hint_size = 18;
        draw_text_centered_screen(
            d,
            hint,
            screen_height / 2 - 32,
            hint_size,
            Color::new(230, 230, 230, 220),
            screen_width,
        );
    }
}
