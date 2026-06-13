use crate::runner::core::TerminalCell;
use super::Bounce;
use super::draw_helpers::{set_cell_helper, draw_string_helper, width_px, height_px};

pub fn draw_system_info_panel(db: &Bounce, grid: &mut [TerminalCell], cols: usize, rows: usize) {
    let blue = db.theme_accent;
    let teal = (0, 240, 200);
    let white = (235, 240, 250);

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
