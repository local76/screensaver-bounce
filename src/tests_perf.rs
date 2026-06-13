use crate::bounce::Bounce;
use crate::runner::core::TerminalCell;
use crate::runner::core::screensaver::Screensaver;
use std::time::{Duration, Instant};

#[test]
fn test_screensaver_perf() {
    let mut b = Bounce::new();
    // Prevent slow system info queries during the test
    b.stat_update_timer = -1000.0;

    let cols = 80;
    let rows = 30;
    let mut grid = vec![TerminalCell::default(); cols * rows];
    let frame_dt = Duration::from_millis(16);

    let start = Instant::now();
    for _ in 0..100 {
        b.update(frame_dt, cols, rows);
        b.draw(&mut grid, cols, rows);
    }
    let elapsed = start.elapsed();

    println!("Performance test took: {:?}", elapsed);
    assert!(
        elapsed < Duration::from_millis(1500),
        "Performance test exceeded budget: {:?}",
        elapsed
    );
}
