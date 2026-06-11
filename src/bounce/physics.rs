use library::core::TerminalCell;
use super::Bounce;
use super::types::{BhopState, CommandState, COMMANDS};

pub fn get_system_info_theme_is_dark() -> bool {
    library::toolkit::sys_info::query_system_theme().is_dark_mode
}

#[allow(clippy::too_many_arguments)]
pub fn set_cell_helper(
    grid: &mut [TerminalCell],
    cols: usize,
    rows: usize,
    x: usize,
    y: usize,
    ch: char,
    fg: (u8, u8, u8),
    bold: bool,
) {
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
pub fn draw_string_helper(
    grid: &mut [TerminalCell],
    cols: usize,
    rows: usize,
    x: usize,
    y: usize,
    text: &str,
    fg: (u8, u8, u8),
    bold: bool,
) {
    for (i, ch) in text.chars().enumerate() {
        set_cell_helper(grid, cols, rows, x + i, y, ch, fg, bold);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_border_helper(
    grid: &mut [TerminalCell],
    cols: usize,
    rows: usize,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    title: &str,
    fg: (u8, u8, u8),
) {
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
        let warn = "Screen size too small for console dashboard.";
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
    let bhop_title = format!(
        "BHOP SIMULATOR v1.3 ({})",
        if db.show_sys_info {
            if is_three_column { "WIDE" } else { "STACK" }
        } else if cols >= 120 {
            "WIDE"
        } else {
            "STACK"
        }
    );
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

        let res_str = format!(
            "{}x{} @ {}Hz (Main Monitor)",
            width_px(cols, db.cell_w),
            height_px(rows, db.cell_h),
            refresh_rate
        );
        draw_string_helper(grid, cols, rows, 4, stats_start_y + 11, "Display: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 11, &res_str, white, false);

        let cpu_id = db.cpu_id.clone();
        draw_string_helper(grid, cols, rows, 4, stats_start_y + 13, "CPU: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 13, &cpu_id, white, false);

        draw_string_helper(grid, cols, rows, 4, stats_start_y + 15, "GPU: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 15, &db.gpus, white, false);

        let ram_pct = (db.ram_used_mb * 100).checked_div(db.ram_total_mb).unwrap_or(0);
        let ram_str = format!(
            "{:.1} GB / {:.1} GB ({}%)",
            db.ram_used_mb as f32 / 1024.0,
            db.ram_total_mb as f32 / 1024.0,
            ram_pct
        );
        draw_string_helper(grid, cols, rows, 4, stats_start_y + 17, "Memory: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 17, &ram_str, white, false);

        draw_string_helper(grid, cols, rows, 4, stats_start_y + 19, "Monitors: ", teal, true);
        draw_string_helper(grid, cols, rows, 14, stats_start_y + 19, &db.monitors, white, false);

        let hex_accent = format!(
            "{} Mode (#{:02X}{:02X}{:02X})",
            db.theme_mode, blue.0, blue.1, blue.2
        );
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
        if gy >= console_y + console_h - 1 {
            break;
        }

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
    let status_x_offset = if bhop_w_content > 68 {
        48
    } else {
        bhop_w_content.saturating_sub(25)
    };
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
    let offset = db.player_y.round().max(0.0) as usize;
    let player_gy = (floor_y - 1).saturating_sub(offset);
    let player_char = match db.bhop_state {
        BhopState::Playing => {
            if db.player_y > 0.1 {
                '☻'
            } else {
                '☺'
            }
        }
        BhopState::Dead | BhopState::Respawning => '☠',
    };
    let p_color = if db.bhop_state == BhopState::Playing {
        blue
    } else {
        orange
    };
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
