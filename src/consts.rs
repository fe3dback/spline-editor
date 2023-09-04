use bevy::prelude::Vec2;

pub const PLOT_WIDTH: f32 = 480.0;
pub const PLOT_HEIGHT: f32 = 240.0;
pub const STATUS_BAR_HEIGHT: f32 = 32.0;

pub const WINDOW_SCALE: f32 = 2.0;
pub const WINDOW_WIDTH: f32 = PLOT_WIDTH * WINDOW_SCALE;
pub const WINDOW_HEIGHT: f32 = PLOT_HEIGHT * WINDOW_SCALE + STATUS_BAR_HEIGHT;
pub const OFFSET: f32 = 10.0;

pub const ACTIVE_RADIUS: f32 = 0.03;

#[inline(always)]
pub fn screen(v: Vec2) -> Vec2 {
    return Vec2::new(
        (v.x * WINDOW_SCALE) - (WINDOW_WIDTH / 2.0),
        (v.y * -WINDOW_SCALE) + (WINDOW_HEIGHT / 2.0),
    );
}

#[inline(always)]
pub fn plot(v_norm: Vec2) -> Vec2 {
    screen(Vec2 {
        x: Vec2::new(OFFSET, 0.0)
            .lerp(Vec2::new(PLOT_WIDTH - OFFSET, 0.0), v_norm.x)
            .x,
        y: Vec2::new(0.0, OFFSET)
            .lerp(Vec2::new(0.0, PLOT_HEIGHT - OFFSET), 1.0 - v_norm.y)
            .y,
    })
}

#[inline(always)]
pub fn roundf32(x: f32, decimals: u32) -> f32 {
    let y = 10i32.pow(decimals) as f32;
    (x * y).round() / y
}
