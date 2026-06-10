//! Consolidated bounce screensaver effect module.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).


use library::core::TerminalCell;
use std::time::Duration;
use library::core::screensaver::Screensaver;
use library::core::logo_block::render_logo_block;


use library::platform::native::sys_info::get_system_info;

use library::apps::identity;
use library::toolkit::sys_info::query_current_palette;


use library::toolkit::rgb_controller::{RgbController, is_openrgb_enabled};


use library::toolkit::rgb_protocol::RgbColor;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CommandState {
    Typing,
    Executing,
    CoolDown,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BhopState {
    Playing,
    Dead,
    Respawning,
}

pub struct LcgRng(u64);
impl LcgRng {
    pub fn new(seed: u64) -> Self {
        Self(seed | 1)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
    pub fn next_f32(&mut self) -> f32 {
        let val = self.next_u64() >> 11;
        (val as f64 / 9007199254740992.0) as f32
    }
    pub fn next_bool(&mut self, prob: f32) -> bool {
        self.next_f32() < prob
    }
}

pub const COMMANDS: &[(&str, &[&str])] = &[
    (
        "ping -c 3 trance-labs.com",
        &[
            "PING trance-labs.com (104.21.32.222): 56 data bytes",
            "64 bytes from 104.21.32.222: icmp_seq=0 ttl=56 time=12.4 ms",
            "64 bytes from 104.21.32.222: icmp_seq=1 ttl=56 time=14.1 ms",
            "64 bytes from 104.21.32.222: icmp_seq=2 ttl=56 time=11.8 ms",
            "--- trance-labs.com ping statistics ---",
            "3 packets transmitted, 3 received, 0% packet loss",
            "rtt min/avg/max = 11.8/12.7/14.1 ms",
        ],
    ),
    (
        "cargo build --release",
        &[
            "   Compiling once_cell v1.19.0",
            "   Compiling windows-sys v0.59.0",
            "   Compiling trance_screensaver v0.1.0",
            "    Finished `release` profile [optimized] target(s) in 3.42s",
        ],
    ),
    (
        "netstat -an | findstr :8080",
        &[
            "  TCP    0.0.0.0:8080           0.0.0.0:0              LISTENING",
            "  TCP    127.0.0.1:8080         127.0.0.1:54932        ESTABLISHED",
            "  TCP    127.0.0.1:54932        127.0.0.1:8080         ESTABLISHED",
        ],
    ),
    (
        "git status",
        &[
            "On branch main",
            "Your branch is up to date with 'origin/main'.",
            "Changes not staged for commit:",
            "  (use \"git add <file>...\" to update what will be committed)",
            "	modified:   src/animation.rs",
            "	modified:   src/renderer.rs",
            "no changes added to commit (use \"git add\" and/or \"git commit -a\")",
        ],
    ),
    (
        "cat /etc/passwd | grep trance",
        &[
            "trance:x:1001:1001:Windows Admin,,,:/home/trance:/bin/bash",
        ],
    ),
];



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
    pub rgb: Option<RgbController>,
    pub last_rgb_color: Option<RgbColor>,
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

        #[cfg(feature = "sys-info")]
        let (username, shell_name, refresh_rate) = (
            identity::username(),
            identity::shell_name(),
            identity::refresh_rate_hz(),
        );
        #[cfg(not(feature = "sys-info"))]
        let (username, shell_name, refresh_rate) = (String::new(), String::new(), 0i32);

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

            console_lines: vec!["Initializing TUI system...".to_string()],
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
            rgb: if is_openrgb_enabled() { Some(RgbController::new()) } else { None },
            last_rgb_color: None,
        }
    }

    fn update_system_stats(&mut self) {
        let sys = get_system_info();
        self.uptime_secs = sys.uptime_secs;
        self.ram_used_mb = sys.mem_used_mb;
        self.ram_total_mb = sys.mem_total_mb;
        self.power_status = sys.power_status;
        self.disk_summary = sys.disk_summary;
        self.gpus = sys.gpus;
        self.monitors = sys.monitors;

        let palette = query_current_palette();
        self.theme_accent = palette.accent;
        #[cfg(feature = "sys-info")]
        {
            self.theme_mode = if get_system_info_theme_is_dark() { "Dark Mode" } else { "Light Mode" }.to_string();
        }
        #[cfg(not(feature = "sys-info"))]
        {
            self.theme_mode = "Default".to_string();
        }
    }
}


fn get_system_info_theme_is_dark() -> bool {
    library::platform::native::sys_info::query_system_theme().is_dark_mode
}

impl Screensaver for Bounce {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        self.cols = cols;
        self.rows = rows;

        let delta = dt.as_secs_f32();
        self.elapsed += delta;

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
                        if let Some(ref r) = self.rgb {
                            r.flash(RgbColor::new(0, 255, 100), Duration::from_millis(150));
                        }
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
                    if let Some(ref r) = self.rgb {
                        r.flash(RgbColor::new(255, 0, 0), Duration::from_millis(500));
                    }
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

        let speed_ratio = (self.bhop_speed / 450.0).clamp(0.1, 1.0);
        let new_color = RgbColor::new(
            (10.0 * speed_ratio) as u8,
            (150.0 * speed_ratio) as u8,
            (255.0 * speed_ratio) as u8,
        );
        if self.last_rgb_color != Some(new_color) {
            self.last_rgb_color = Some(new_color);
            if let Some(ref r) = self.rgb {
                r.set_color(new_color);
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


#[allow(clippy::too_many_arguments)]
pub fn set_cell_helper(grid: &mut [TerminalCell], cols: usize, rows: usize, x: usize, y: usize, ch: char, fg: (u8, u8, u8), bold: bool) {
    if x < cols && y < rows {
        let idx = y * cols + x;
        grid[idx] = TerminalCell {
            ch,
            fg,
            bg: (0, 0, 0),
            bold,
        };
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_string_helper(grid: &mut [TerminalCell], cols: usize, rows: usize, x: usize, y: usize, text: &str, fg: (u8, u8, u8), bold: bool) {
    for (i, ch) in text.chars().enumerate() {
        set_cell_helper(grid, cols, rows, x + i, y, ch, fg, bold);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_border_helper(grid: &mut [TerminalCell], cols: usize, rows: usize, x: usize, y: usize, w: usize, h: usize, title: &str, fg: (u8, u8, u8)) {
    set_cell_helper(grid, cols, rows, x, y, '╔', fg, false);
    set_cell_helper(grid, cols, rows, x + w - 1, y, '╗', fg, false);
    set_cell_helper(grid, cols, rows, x, y + h - 1, '╚', fg, false);
    set_cell_helper(grid, cols, rows, x + w - 1, y + h - 1, '╝', fg, false);

    for cx in (x + 1)..(x + w - 1) {
        set_cell_helper(grid, cols, rows, cx, y, '═', fg, false);
        set_cell_helper(grid, cols, rows, cx, y + h - 1, '═', fg, false);
    }
    for cy in (y + 1)..(y + h - 1) {
        set_cell_helper(grid, cols, rows, x, cy, '║', fg, false);
        set_cell_helper(grid, cols, rows, x + w - 1, cy, '║', fg, false);
    }

    if !title.is_empty() {
        let title_str = format!(" {} ", title);
        let title_chars: Vec<char> = title_str.chars().collect();
        let tx = x + (w - title_chars.len()) / 2;
        for (i, &ch) in title_chars.iter().enumerate() {
            set_cell_helper(grid, cols, rows, tx + i, y, ch, fg, true);
        }
    }
}

pub fn width_px(cols: usize, cell_w: i32) -> i32 {
    (cols as i32) * cell_w
}

pub fn height_px(rows: usize, cell_h: i32) -> i32 {
    (rows as i32) * cell_h
}

pub fn draw_dashboard(db: &Bounce, grid: &mut [TerminalCell], cols: usize, rows: usize) {
    let default_cell = TerminalCell {
        ch: ' ',
        fg: db.theme_accent,
        bg: (0, 0, 0),
        bold: false,
    };
    for cell in grid.iter_mut() {
        *cell = default_cell;
    }

    if cols < 80 || rows < 30 {
        let warn = "Screen size too small for TUI Dashboard.";
        let start_x = cols.saturating_sub(warn.len()) / 2;
        let start_y = rows / 2;
        for (i, ch) in warn.chars().enumerate() {
            set_cell_helper(grid, cols, rows, start_x + i, start_y, ch, (255, 120, 0), false);
        }
        return;
    }

    let blue = db.theme_accent;
    let teal = (0, 240, 200);
    let white = (235, 240, 250);
    let orange = (255, 120, 0);

    let is_three_column = cols >= 220;

    let (console_x, console_y, console_w, console_h);
    let (bhop_x, bhop_y, bhop_w, bhop_h);

    if db.show_sys_info {
        if is_three_column {
            let total_rem_w = cols - 76;
            let usable_w = total_rem_w.saturating_sub(1);
            let col2_w = usable_w / 2;
            let col3_w = usable_w - col2_w;

            console_x = 76;
            console_y = 0;
            console_w = col2_w;
            console_h = rows - 2;

            bhop_x = 76 + col2_w + 1;
            bhop_y = 0;
            bhop_w = col3_w;
            bhop_h = rows - 2;
        } else {
            console_x = 76;
            console_y = 0;
            console_w = cols - 77;
            console_h = (rows - 2) / 2 + 1;

            bhop_x = 76;
            bhop_y = console_h;
            bhop_w = cols - 77;
            bhop_h = rows - 2 - bhop_y;
        }

        // Panel 1: System Info
        draw_border_helper(grid, cols, rows, 0, 0, 75, rows - 2, "SYSTEM DIAGNOSTICS", blue);
    } else {
        if cols >= 120 {
            let usable_w = cols.saturating_sub(1);
            let col2_w = usable_w / 2;
            let col3_w = usable_w - col2_w;

            console_x = 0;
            console_y = 0;
            console_w = col2_w;
            console_h = rows - 2;

            bhop_x = col2_w + 1;
            bhop_y = 0;
            bhop_w = col3_w;
            bhop_h = rows - 2;
        } else {
            console_x = 0;
            console_y = 0;
            console_w = cols;
            console_h = (rows - 2) / 2 + 1;

            bhop_x = 0;
            bhop_y = console_h;
            bhop_w = cols;
            bhop_h = rows - 2 - bhop_y;
        }
    }
    
    // Panel 2: Command Console
    draw_border_helper(grid, cols, rows, console_x, console_y, console_w, console_h, "COMMAND CONSOLE", blue);

    // Panel 3: Bhop Game
    let bhop_title = format!("BHOP SIMULATOR v1.3 ({})", if db.show_sys_info { if is_three_column { "WIDE" } else { "STACK" } } else { if cols >= 120 { "WIDE" } else { "STACK" } });
    draw_border_helper(grid, cols, rows, bhop_x, bhop_y, bhop_w, bhop_h, &bhop_title, blue);

    // --- PANEL 1 CONTENTS ---
    if db.show_sys_info {
        for (r, line) in db.logo_lines.iter().enumerate() {
            let gy = 2 + r;
            for (c, ch) in line.chars().enumerate() {
                let gx = 4 + c;
                if ch != ' ' {
                    set_cell_helper(grid, cols, rows, gx, gy, ch, blue, true);
                }
            }
        }

        let divider_y = 9;
        for x in 2..73 {
            set_cell_helper(grid, cols, rows, x, divider_y, '─', (60, 60, 75), false);
        }

        let stats_start_y = 11;
        let user_title = format!("{}@{}", db.username, db.hostname.to_lowercase());
        let os_name = db.os_name.clone();
        let kernel_version = db.kernel_version.clone();
        let shell_name = db.shell_name.clone();
        let refresh_rate = db.refresh_rate;

        draw_string_helper(grid, cols, rows, 4, stats_start_y, &user_title, blue, true);
        draw_string_helper(grid, cols, rows, 4, stats_start_y + 1, "--------------------------------------------", teal, false);

        draw_string_helper(grid, cols, rows, 4, stats_start_y + 3, "BUILD: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 3, &os_name, white, false);

        draw_string_helper(grid, cols, rows, 4, stats_start_y + 5, "Kernel: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 5, &kernel_version, white, false);

        let h = db.uptime_secs / 3600;
        let m = (db.uptime_secs % 3600) / 60;
        let s = db.uptime_secs % 60;
        let uptime_str = format!("{}h {}m {}s", h, m, s);
        draw_string_helper(grid, cols, rows, 4, stats_start_y + 7, "Uptime: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 7, &uptime_str, white, false);

        draw_string_helper(grid, cols, rows, 4, stats_start_y + 9, "Shell: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 9, &shell_name, white, false);

        let res_str = format!("{}x{} @ {}Hz (Main Monitor)", width_px(cols, db.cell_w), height_px(rows, db.cell_h), refresh_rate);
        draw_string_helper(grid, cols, rows, 4, stats_start_y + 11, "Display: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 11, &res_str, white, false);

        let cpu_id = db.cpu_id.clone();
        draw_string_helper(grid, cols, rows, 4, stats_start_y + 13, "CPU: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 13, &cpu_id, white, false);

        draw_string_helper(grid, cols, rows, 4, stats_start_y + 15, "GPU: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 15, &db.gpus, white, false);

        let ram_pct = (db.ram_used_mb * 100).checked_div(db.ram_total_mb).unwrap_or(0);
        let ram_str = format!("{:.1} GB / {:.1} GB ({}%)", db.ram_used_mb as f32 / 1024.0, db.ram_total_mb as f32 / 1024.0, ram_pct);
        draw_string_helper(grid, cols, rows, 4, stats_start_y + 17, "Memory: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 17, &ram_str, white, false);

        draw_string_helper(grid, cols, rows, 4, stats_start_y + 19, "Monitors: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 19, &db.monitors, white, false);

        let hex_accent = format!("{} Mode (#{:02X}{:02X}{:02X})", db.theme_mode, blue.0, blue.1, blue.2);
        draw_string_helper(grid, cols, rows, 4, stats_start_y + 21, "Theme: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 21, &hex_accent, white, false);

        draw_string_helper(grid, cols, rows, 4, stats_start_y + 23, "Power: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 23, &db.power_status, white, false);

        draw_string_helper(grid, cols, rows, 4, stats_start_y + 25, "Disk: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 25, &db.disk_summary, white, false);

        draw_string_helper(grid, cols, rows, 4, stats_start_y + 27, "AI Skill: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 27, &format!("{:.0}%", db.auto_skill * 100.0), white, false);

        let blocks_y = rows.saturating_sub(5);
        let colors = [
            (255, 0, 127),
            (0, 255, 255),
            (0, 180, 255),
            (255, 255, 0),
            (240, 240, 255),
            (60, 60, 75),
        ];
        if blocks_y >= stats_start_y + 19 {
            for (i, &col) in colors.iter().enumerate() {
                for j in 0..6 {
                    set_cell_helper(grid, cols, rows, 4 + i * 8 + j, blocks_y, '█', col, false);
                    set_cell_helper(grid, cols, rows, 4 + i * 8 + j, blocks_y + 1, '█', col, false);
                }
            }
        }
    }

    // --- PANEL 2 CONTENTS (Console) ---
    let console_start_x = console_x + 2;
    let console_start_y = console_y + 1;
    let cursor_visible = (db.elapsed % 0.6) < 0.3;

    let console_visible_rows = (console_h as i32 - 2).max(1) as usize;
    let start_idx = db.console_lines.len().saturating_sub(console_visible_rows);
    let lines_to_draw: Vec<String> = db.console_lines[start_idx..].to_vec();

    for (row_idx, line) in lines_to_draw.iter().enumerate() {
        let gy = console_start_y + row_idx;
        if gy >= console_y + console_h - 1 { break; }

        if row_idx == lines_to_draw.len() - 1 && db.command_state == CommandState::Typing {
            let full_cmd = COMMANDS[db.current_command_idx].0;
            let typed_part = &full_cmd[0..db.current_typed_len];
            let typed_line = format!("{}{}", line, typed_part);
            
            let max_len = console_w.saturating_sub(4);
            let display_line: String = typed_line.chars().take(max_len).collect();
            
            draw_string_helper(grid, cols, rows, console_start_x, gy, &display_line, teal, false);
            if cursor_visible {
                let cur_x = console_start_x + display_line.chars().count();
                if cur_x < console_x + console_w - 1 {
                    set_cell_helper(grid, cols, rows, cur_x, gy, '█', teal, true);
                }
            }
        } else {
            let is_prompt = line.starts_with(&format!("{}@", db.username));
            let color = if is_prompt { blue } else { white };
            
            let max_len = console_w.saturating_sub(4);
            let display_line: String = line.chars().take(max_len).collect();
            
            draw_string_helper(grid, cols, rows, console_start_x, gy, &display_line, color, is_prompt);
            
            if row_idx == lines_to_draw.len() - 1 && db.command_state == CommandState::CoolDown && cursor_visible {
                let cur_x = console_start_x + display_line.chars().count();
                if cur_x < console_x + console_w - 1 {
                    set_cell_helper(grid, cols, rows, cur_x, gy, '█', blue, true);
                }
            }
        }
    }

    // --- PANEL 3 CONTENTS (Bhop Game) ---
    let bhop_start_x = bhop_x + 2;
    let bhop_start_y = bhop_y + 1;
    let bhop_w_content = bhop_w.saturating_sub(4);
    let bhop_panel_h = bhop_h - 2;

    let bhop_score_str = format!("SCORE: {:<4}", db.bhop_score);
    let bhop_best_str = format!("BEST: {:<4}", db.bhop_best);
    let bhop_speed_str = format!("SPEED: {:.0} u/s", db.bhop_speed);
    
    draw_string_helper(grid, cols, rows, bhop_start_x, bhop_start_y, &bhop_score_str, teal, true);
    draw_string_helper(grid, cols, rows, bhop_start_x + 15, bhop_start_y, &bhop_best_str, teal, true);
    draw_string_helper(grid, cols, rows, bhop_start_x + 30, bhop_start_y, &bhop_speed_str, teal, false);

    let (status_text, status_color) = match db.bhop_state {
        BhopState::Playing => ("STATUS: BUNNY HOPPING", teal),
        BhopState::Dead => ("STATUS: GAME OVER (CRASHED)", orange),
        BhopState::Respawning => ("STATUS: RE-SPAWNING...", (255, 255, 0)),
    };
    let status_x_offset = if bhop_w_content > 68 { 48 } else { bhop_w_content.saturating_sub(25) };
    if bhop_start_x + status_x_offset + status_text.len() < bhop_x + bhop_w {
        draw_string_helper(grid, cols, rows, bhop_start_x + status_x_offset, bhop_start_y, status_text, status_color, true);
    }

    for x in bhop_start_x..(bhop_start_x + bhop_w_content) {
        set_cell_helper(grid, cols, rows, x, bhop_start_y + 2, '═', (60, 60, 75), false);
    }

    let game_h = bhop_panel_h - 3;
    let floor_y = bhop_start_y + 3 + game_h - 2;
    
    for x in bhop_start_x..(bhop_start_x + bhop_w_content) {
        set_cell_helper(grid, cols, rows, x, floor_y, '▀', (60, 60, 75), false);
    }

    let player_gx = bhop_start_x + 6;
    let player_gy = floor_y - 1 - db.player_y.round() as usize;
    let player_char = match db.bhop_state {
        BhopState::Playing => {
            if db.player_y > 0.1 { '☻' } else { '☺' }
        }
        BhopState::Dead | BhopState::Respawning => '☠',
    };
    let p_color = if db.bhop_state == BhopState::Playing { blue } else { orange };
    if player_gy < rows {
        set_cell_helper(grid, cols, rows, player_gx, player_gy, player_char, p_color, true);
    }

    if db.bhop_state == BhopState::Playing {
        let obs_gx = bhop_start_x + db.obstacle_x.round() as usize;
        if obs_gx < bhop_start_x + bhop_w_content {
            set_cell_helper(grid, cols, rows, obs_gx, floor_y - 1, '▲', teal, true);
        }
    }

    let scenery_y = floor_y - 4;
    let scenery_offset = ((db.elapsed * 10.0) as usize) % bhop_w_content.max(1);
    for i in 0..bhop_w_content {
        if (i + scenery_offset) % 15 == 0 {
            set_cell_helper(grid, cols, rows, bhop_start_x + i, scenery_y, '.', (60, 60, 75), false);
        }
        if (i + scenery_offset + 5) % 25 == 0 {
            set_cell_helper(grid, cols, rows, bhop_start_x + i, scenery_y - 2, '*', (40, 40, 50), false);
        }
    }
}
