use crate::runner::core::TerminalCell;
use super::Bounce;
use super::types::{CommandState, COMMANDS};
use super::draw_helpers::{draw_string_helper, set_cell_helper};

pub fn draw_console_panel(
    db: &Bounce,
    grid: &mut [TerminalCell],
    cols: usize,
    rows: usize,
    console_x: usize,
    console_y: usize,
    console_w: usize,
    console_h: usize,
    blue: (u8, u8, u8),
    teal: (u8, u8, u8),
) {
    let white = (235, 240, 250);
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
}
