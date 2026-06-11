//! Consolidated bounce screensaver effect module.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).

use library::core::TerminalCell;
use std::time::Duration;
use library::core::screensaver::Screensaver;
use library::core::logo_block::render_logo_block;

use library::toolkit::sys_info::get_system_info;
use library::apps::identity;
use library::toolkit::sys_info::query_current_palette;



pub mod types;
pub mod physics;

use types::{BhopState, CommandState, LcgRng, COMMANDS};
use physics::{draw_dashboard, get_system_info_theme_is_dark};

pub struct Bounce {
    pub cols: usize,
    pub rows: usize,
    pub cell_w: i32,
    pub cell_h: i32,
    pub theme_mode: String,
    pub show_sys_info: bool,
    pub speed_opt: u32,

    // System stats
    pub hostname: String,
    pub username: String,
    pub cpu_id: String,
    pub os_name: String,
    pub kernel_version: String,
    pub shell_name: String,
    pub refresh_rate: i32,
    pub theme_accent: (u8, u8, u8),
    pub uptime_secs: u64,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub stat_update_timer: f32,
    pub power_status: String,
    pub disk_summary: String,
    pub gpus: String,
    pub monitors: String,
    pub auto_skill: f32,
    pub logo_lines: Vec<String>,

    // Command Console state
    pub console_lines: Vec<String>,
    pub current_command_idx: usize,
    pub current_typed_len: usize,
    pub command_state: CommandState,
    pub command_timer: f32,

    // Bhop Game state
    pub player_y: f32,
    pub player_vy: f32,
    pub obstacle_x: f32,
    pub bhop_score: usize,
    pub bhop_best: usize,
    pub bhop_speed: f32,
    pub bhop_state: BhopState,
    pub bhop_timer: f32,

    pub elapsed: f32,
    pub(crate) rng: LcgRng,
    pub(super) on_battery: bool,
    pub(super) frame_time_ema: f32,
    pub(super) quality_scale: f32,
    pub(super) target_frame_time: f32,
}

impl Default for Bounce {
    fn default() -> Self {
        Self::new()
    }
}

impl Bounce {
    pub fn new() -> Self {
        // Pre-4.1 Windows-only cell sizing (GetDC/GetDeviceCaps) collapsed
        // to a sane default. The dpi-aware cell sizing returns in 4.2
        // alongside the screensaver_runtime move. Default: 12x20 px cells.
        let cell_w: i32 = 12;
        let cell_h: i32 = 20;

        let sys = get_system_info();
        let hostname = sys.hostname;
        let cpu_id = sys.cpu;
        let os_name = sys.os;
        let kernel_version = sys.kernel;

        let (username, shell_name, refresh_rate) = (
            identity::username(),
            identity::shell_name(),
            identity::refresh_rate_hz(),
        );

        // library 4.0: pull the accent from the canonical ScreenPalette.
        let theme_accent = query_current_palette().accent;
        let theme_mode = if get_system_info_theme_is_dark() { "Dark Mode" } else { "Light Mode" }.to_string();

        // Pre-4.1 HKEY_CURRENT_USER registry reads (Speed, ShowSysInfo) collapsed
        // to defaults. Re-added in 4.2.
        let speed_opt: u32 = 1;
        let show_sys_info: bool = true;

        let bhop_speed = match speed_opt {
            0 => 150.0,
            2 => 400.0,
            _ => 250.0,
        };

        let logo_lines = render_logo_block(&sys.logo_text, None);
        let on_battery = sys.power_status.contains("Battery");

        Self {
            cols: 80,
            rows: 30,
            cell_w,
            cell_h,
            theme_mode,
            show_sys_info,
            speed_opt,
            hostname,
            username,
            cpu_id,
            os_name,
            kernel_version,
            shell_name,
            refresh_rate,
            theme_accent,
            uptime_secs: sys.uptime_secs,
            ram_used_mb: sys.mem_used_mb,
            ram_total_mb: sys.mem_total_mb,
            stat_update_timer: 9.0,
            power_status: sys.power_status.clone(),
            disk_summary: sys.disk_summary.clone(),
            gpus: sys.gpus.clone(),
            monitors: sys.monitors.clone(),
            auto_skill: 0.72,
            logo_lines,

            console_lines: vec!["Initializing console system...".to_string()],
            current_command_idx: 0,
            current_typed_len: 0,
            command_state: CommandState::CoolDown,
            command_timer: 0.0,

            player_y: 0.0,
            player_vy: 0.0,
            obstacle_x: 40.0,
            bhop_score: 0,
            bhop_best: 0,
            bhop_speed,
            bhop_state: BhopState::Playing,
            bhop_timer: 0.0,

            elapsed: 0.0,
            rng: LcgRng::new(9876),
            on_battery,
            frame_time_ema: 0.01666667,
            quality_scale: 1.0,
            target_frame_time: 0.01666667,
        }
    }

    fn update_system_stats(&mut self) {
        let sys = get_system_info();
        self.uptime_secs = sys.uptime_secs;
        self.ram_used_mb = sys.mem_used_mb;
        self.ram_total_mb = sys.mem_total_mb;
        self.power_status = sys.power_status.clone();
        self.on_battery = sys.power_status.contains("Battery");
        self.disk_summary = sys.disk_summary;
        self.gpus = sys.gpus;
        self.monitors = sys.monitors;

        let palette = query_current_palette();
        self.theme_accent = palette.accent;
        self.theme_mode = if get_system_info_theme_is_dark() { "Dark Mode" } else { "Light Mode" }.to_string();
    }
}

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

                // Pre-4.1 Windows GetAsyncKeyState(VK_SPACE) keyboard jump
                // dropped from the inline migration. Will be re-added in
                // 4.2 alongside the screensaver_runtime's native input layer.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounce_new() {
        let b = Bounce::new();
        assert_eq!(b.cols, 80);
        assert_eq!(b.rows, 30);
    }

    #[test]
    fn test_bounce_update_and_draw() {
        let mut b = Bounce::new();
        b.update(Duration::from_millis(16), 80, 24);
        let mut grid = vec![TerminalCell::default(); 80 * 24];
        b.draw(&mut grid, 80, 24);
        // Ensure some characters were drawn or it completed without panic
        assert_eq!(b.cols, 80);
        assert_eq!(b.rows, 24);
    }
}

