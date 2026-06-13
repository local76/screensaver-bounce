use crate::runner::core::TerminalCell;
use crate::runner::core::screensaver::Screensaver;
use std::time::Duration;
use super::Bounce;
use super::types::{BhopState, CommandState, COMMANDS};
use super::physics::draw_dashboard;

impl Screensaver for Bounce {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        self.cols = cols;
        self.rows = rows;

        let dt_secs = dt.as_secs_f32();

        // Auto-detect high refresh rates during the startup phase
        if self.elapsed < 2.0 && dt_secs > 0.001 {
            if dt_secs < self.target_frame_time - 0.001 {
                self.target_frame_time = dt_secs;
            }
        }

        // Exponential moving average for frame time (alpha = 0.1)
        self.frame_time_ema = self.frame_time_ema * 0.9 + dt_secs.min(0.2) * 0.1;

        let speed_mult = if self.on_battery { 0.65 } else { 1.0 };
        let delta = dt_secs * speed_mult;
        self.elapsed += delta;

        // Adjust quality_scale based on frame time performance vs target
        if self.elapsed > 1.5 {
            if self.frame_time_ema > self.target_frame_time * 1.15 {
                self.quality_scale = (self.quality_scale - 0.15 * delta).max(0.20);
            } else if self.frame_time_ema < self.target_frame_time * 1.05 {
                self.quality_scale = (self.quality_scale + 0.04 * delta).min(1.0);
            }
        }

        self.stat_update_timer += delta;
        if self.stat_update_timer >= 1.0 {
            self.update_system_stats();
            self.stat_update_timer = 0.0;
        }

        self.command_timer += delta;
        let command = COMMANDS[self.current_command_idx];
        match self.command_state {
            CommandState::Typing => {
                if self.command_timer >= 0.05 {
                    self.current_typed_len += 1;
                    self.command_timer = 0.0;
                    if self.current_typed_len >= command.0.len() {
                        self.command_state = CommandState::Executing;
                        self.command_timer = 0.0;
                    }
                }
            }
            CommandState::Executing => {
                if self.command_timer >= 0.15 {
                    let total_typed_lines = self.console_lines.len();
                    if total_typed_lines > 0 {
                        let mut printed_count = 0;
                        for line in self.console_lines.iter().rev() {
                            if line.starts_with(&format!("{}@", self.username)) && line.contains("$ ") {
                                break;
                            }
                            printed_count += 1;
                        }

                        if printed_count < command.1.len() {
                            let next_line = command.1[printed_count].to_string();
                            self.console_lines.push(next_line);
                            if self.console_lines.len() > 100 {
                                self.console_lines.remove(0);
                            }
                        } else {
                            self.command_state = CommandState::CoolDown;
                            self.command_timer = 0.0;
                        }
                    }
                    self.command_timer = 0.0;
                }
            }
            CommandState::CoolDown => {
                if self.command_timer >= 2.0 {
                    self.current_command_idx = (self.current_command_idx + 1) % COMMANDS.len();
                    self.current_typed_len = 0;
                    self.command_state = CommandState::Typing;
                    self.command_timer = 0.0;

                    let prompt = format!("{}@{}:~$ ", self.username, self.hostname.to_lowercase());
                    self.console_lines.push(prompt);
                    if self.console_lines.len() > 100 {
                        self.console_lines.remove(0);
                    }
                }
            }
        }

        self.bhop_timer += delta;
        match self.bhop_state {
            BhopState::Playing => {
                let speed_multiplier = 0.12 * (self.bhop_speed / 250.0);
                self.obstacle_x -= self.bhop_speed * speed_multiplier * delta;

                let bhop_w = if self.show_sys_info {
                    if cols >= 220 {
                        let total_rem_w = cols.saturating_sub(76);
                        let usable_w = total_rem_w.saturating_sub(1);
                        let col2_w = usable_w / 2;
                        usable_w - col2_w
                    } else {
                        cols.saturating_sub(77)
                    }
                } else {
                    if cols >= 120 {
                        let usable_w = cols.saturating_sub(1);
                        let col2_w = usable_w / 2;
                        usable_w - col2_w
                    } else {
                        cols
                    }
                };
                let bhop_w_content = bhop_w.saturating_sub(4);
                let max_obs_x = (bhop_w_content as f32 - 4.0).max(40.0);

                if self.obstacle_x <= 0.0 {
                    self.obstacle_x = max_obs_x;
                    self.bhop_score += 1;
                    let max_speed = match self.speed_opt {
                        0 => 300.0,
                        2 => 650.0,
                        _ => 450.0,
                    };
                    self.bhop_speed = (self.bhop_speed + 8.0).min(max_speed);
                }

                let player_x = 6.0f32;
                let trigger_dist = 6.8f32 - self.auto_skill * 1.2;
                if self.obstacle_x < player_x + trigger_dist && self.obstacle_x > player_x && self.player_y <= 0.1 {
                    let jump_prob = 0.90 + self.auto_skill * 0.09;
                    if self.rng.next_bool(jump_prob) {
                        self.player_vy = 12.0 + self.auto_skill * 2.0;
                        self.auto_skill = (self.auto_skill + 0.003).min(0.98);
                    }
                }

                if self.player_y > 0.0 || self.player_vy > 0.0 {
                    self.player_y += self.player_vy * delta * 4.0;
                    self.player_vy -= 26.0 * delta * 4.0;
                    if self.player_y <= 0.0 {
                        self.player_y = 0.0;
                        self.player_vy = 0.0;
                    }
                }

                let player_x_int = player_x.round() as i32;
                let obs_x_int = self.obstacle_x.round() as i32;
                if obs_x_int == player_x_int && self.player_y < 1.0 {
                    self.bhop_state = BhopState::Dead;
                    self.bhop_timer = 0.0;
                    self.bhop_speed = 0.0;
                    self.auto_skill = (self.auto_skill * 0.92).max(0.65);
                }
            }
            BhopState::Dead => {
                if self.bhop_timer >= 2.0 {
                    self.bhop_state = BhopState::Respawning;
                    self.bhop_timer = 0.0;
                }
            }
            BhopState::Respawning => {
                if self.bhop_timer >= 1.5 {
                    if self.bhop_score > self.bhop_best {
                        self.bhop_best = self.bhop_score;
                    }
                    self.bhop_score = 0;
                    self.bhop_speed = match self.speed_opt {
                        0 => 150.0,
                        2 => 400.0,
                        _ => 250.0,
                    };
                    if self.bhop_best > 5 {
                        self.auto_skill = (self.auto_skill + 0.05).min(0.98);
                    }

                    let bhop_w = if self.show_sys_info {
                        if cols >= 220 {
                            let total_rem_w = cols.saturating_sub(76);
                            let usable_w = total_rem_w.saturating_sub(1);
                            let col2_w = usable_w / 2;
                            usable_w - col2_w
                        } else {
                            cols.saturating_sub(77)
                        }
                    } else {
                        if cols >= 120 {
                            let usable_w = cols.saturating_sub(1);
                            let col2_w = usable_w / 2;
                            usable_w - col2_w
                        } else {
                            cols
                        }
                    };
                    let bhop_w_content = bhop_w.saturating_sub(4);
                    let max_obs_x = (bhop_w_content as f32 - 4.0).max(40.0);

                    self.obstacle_x = max_obs_x;
                    self.player_y = 0.0;
                    self.player_vy = 0.0;
                    self.bhop_state = BhopState::Playing;
                    self.bhop_timer = 0.0;
                }
            }
        }
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        draw_dashboard(self, grid, cols, rows);
    }

    fn has_scanlines(&self) -> bool {
        true
    }
}
