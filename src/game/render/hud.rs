use raylib::prelude::{Color, RaylibDraw, RaylibDrawHandle};

use crate::config::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::entities::{Tank, Team};

use super::Game;

impl Game {
    pub(super) fn draw_hud(&self, d: &mut RaylibDrawHandle) {
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

    pub(super) fn draw_round_over(&self, d: &mut RaylibDrawHandle) {
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
