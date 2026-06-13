//! Consolidated bounce screensaver effect module.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).

use crate::runner::core::logo_block::render_logo_block;

use crate::runner::toolkit::sys_info::get_system_info;
use crate::runner::apps::identity;
use crate::runner::toolkit::sys_info::query_current_palette;

pub mod types;
pub mod physics;
pub mod draw_helpers;
pub mod panel_sys_info;
pub mod panel_console;
pub mod screensaver_impl;

use types::{BhopState, CommandState, LcgRng};
use draw_helpers::get_system_info_theme_is_dark;

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

#[cfg(test)]
#[path = "bounce_tests.rs"]
mod tests;
