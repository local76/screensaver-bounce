use crate::runner::core::TerminalCell;

pub fn get_system_info_theme_is_dark() -> bool {
    crate::runner::toolkit::sys_info::query_system_theme().is_dark_mode
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
        if w >= title_chars.len() {
            let tx = x + (w - title_chars.len()) / 2;
            for (i, &ch) in title_chars.iter().enumerate() {
                set_cell_helper(grid, cols, rows, tx + i, y, ch, fg, true);
            }
        }
    }
}

pub fn width_px(cols: usize, cell_w: i32) -> i32 {
    (cols as i32) * cell_w
}

pub fn height_px(rows: usize, cell_h: i32) -> i32 {
    (rows as i32) * cell_h
}
