use std::time::Duration;
use crate::runner::core::TerminalCell;
use crate::runner::core::screensaver::Screensaver;
use super::*;
use super::types::LcgRng;
use super::draw_helpers::{width_px, height_px};

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

// Math tests as required by the checklist
#[test]
fn test_lcg_rng_bounds() {
    let mut rng = LcgRng::new(12345);
    for _ in 0..1000 {
        let val = rng.next_f32();
        assert!(val >= 0.0);
        assert!(val < 1.0);
    }
}

#[test]
fn test_lcg_rng_seeding() {
    // Check that even seed produces a valid odd seed internally (due to `seed | 1`)
    let mut rng1 = LcgRng::new(42);
    let mut rng2 = LcgRng::new(43);
    // 42 | 1 is 43, 43 | 1 is 43, so they should produce the exact same sequence
    assert_eq!(rng1.next_u64(), rng2.next_u64());
}

#[test]
fn test_lcg_rng_bool_probability() {
    let mut rng = LcgRng::new(9876);
    let mut true_count = 0;
    let iterations = 1000;
    for _ in 0..iterations {
        if rng.next_bool(0.5) {
            true_count += 1;
        }
    }
    // With 1000 trials, probability of 0.5 should be within range [400, 600]
    assert!(true_count >= 400 && true_count <= 600, "True count: {}", true_count);
}

#[test]
fn test_coordinate_math() {
    assert_eq!(width_px(80, 12), 960);
    assert_eq!(width_px(0, 12), 0);
    assert_eq!(height_px(30, 20), 600);
    assert_eq!(height_px(0, 20), 0);
}
