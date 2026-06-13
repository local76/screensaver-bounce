use crate::runner::core::TerminalCell;
use super::Bounce;
use super::types::BhopState;
use super::draw_helpers::{
    draw_border_helper, draw_string_helper, set_cell_helper,
};
use super::panel_sys_info::draw_system_info_panel;
use super::panel_console::draw_console_panel;

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
        draw_system_info_panel(db, grid, cols, rows);
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
    draw_console_panel(db, grid, cols, rows, console_x, console_y, console_w, console_h, blue, teal);

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
    draw_bhop_panel(db, grid, cols, rows, bhop_x, bhop_y, bhop_w, bhop_h, blue, teal);
}

fn draw_bhop_panel(
    db: &Bounce,
    grid: &mut [TerminalCell],
    cols: usize,
    rows: usize,
    bhop_x: usize,
    bhop_y: usize,
    bhop_w: usize,
    bhop_h: usize,
    blue: (u8, u8, u8),
    teal: (u8, u8, u8),
) {
    let orange = (255, 120, 0);
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
