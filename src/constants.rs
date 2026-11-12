//! Movement in the project's 80-unit grid, seconds, and units per second.
//!
//! Tune movement here. Distances use world units, time uses seconds, angles use
//! degrees. Positive rotation is counterclockwise in the Rust world (Y up).

// Simulation and world. CEILING is an absolute world Y, not C++ CEILING_HEIGHT.
pub const HZ: f64 = 240.;
pub const DT: f64 = 1. / HZ;
// Native GD uses a 30-unit block and a 60 Hz delta scaled by 0.9 for Y.
// Keep the existing 80-unit level files, converting physics in one place.
pub const GD_SCALE: f64 = 80. / 30.;
pub const GD_TIME_SCALE: f64 = 54.;
pub const GD_VELOCITY_UNIT: f64 = GD_TIME_SCALE * GD_SCALE;
pub const SPEED: f64 = 5.770002 * 54. * GD_SCALE;
pub const CEILING: f64 = GROUND_HEIGHT_FOR_WAVE_AND_SHIP + 10. * BLOCK_SIZE;
pub const VIEWPORT_HEIGHT: f64 = 320. * GD_SCALE;
pub const GAME_CAMERA_Y: f64 = GROUND_HEIGHT_FOR_CUBE + VIEWPORT_HEIGHT / 2. - 90. * GD_SCALE;
pub const PLAYER_SCREEN_X: f64 = 210. * GD_SCALE;
pub const CUBE_SIZE: f64 = 80.;
pub const BLOCK_SIZE: f64 = CUBE_SIZE;
pub const GRID_SIZE: f64 = BLOCK_SIZE;
pub const SHIP_SIZE: f64 = CUBE_SIZE;
pub const WAVE_WIDTH: f64 = 10. * GD_SCALE;
pub const WAVE_HEIGHT: f64 = 10. * GD_SCALE;

// Cube. Keep GRAVITY/JUMP as the editable names used by existing configurations.
pub const GRAVITY: f64 = 0.958199 * GD_TIME_SCALE * GD_VELOCITY_UNIT;
pub const JUMP: f64 = 11.18 * GD_VELOCITY_UNIT;
pub const MAX_FALL_SPEED: f64 = 15. * GD_VELOCITY_UNIT;
pub const CUBE_ROTATION_SPEED: f64 = 180. / (26. / 60.);
pub const CUBE_ROTATION_LERP_SPEED: f64 = 28.35;
pub const ROTATION_ALIGNMENT_STEP: f64 = 90.;

// Ship acceleration changes near the apex; rise and fall caps differ.
pub const SHIP_GRAVITY: f64 = GRAVITY;
pub const SHIP_MAX_RISE_SPEED: f64 = 8. * GD_VELOCITY_UNIT;
pub const SHIP_MAX_FALL_SPEED: f64 = 6.4 * GD_VELOCITY_UNIT;
pub const SHIP_ROTATION_LERP: f64 = 9.;
pub const SHIP_MAX_ANGLE: f64 = 80.;

pub const WAVE_SPEED: f64 = SPEED;
pub const WAVE_ANGLE: f64 = 45.;
pub const WAVE_ROTATION_LERP: f64 = 15.;

// Automatic pads and manually activated orbs have independent impulses.
pub const ORB_JUMP_FORCE: f64 = JUMP;
pub const JUMP_PAD_CUBE_FORCE: f64 = 16. * GD_VELOCITY_UNIT;
pub const JUMP_PAD_SHIP_FORCE: f64 = JUMP_PAD_CUBE_FORCE;

// These additional modes were introduced by the Rust port.
pub const ROBOT_JUMP: f64 = JUMP * 0.5;
pub const ROBOT_GRAVITY: f64 = GRAVITY * 0.9;
pub const ROBOT_HOLD: f64 = 15. / GD_TIME_SCALE;
pub const ROBOT_HOLD_GRAVITY_FACTOR: f64 = 0.;
pub const ROBOT_ROTATION_SPEED: f64 = 0.;
pub const ROBOT_ROTATION_LERP_SPEED: f64 = 20.;
pub const BALL_GRAVITY: f64 = GRAVITY * 0.6;
pub const BALL_ROTATION_SPEED: f64 = CUBE_ROTATION_SPEED;
pub const SPIDER_GRAVITY: f64 = BALL_GRAVITY;

// Player and object collision geometry. Small inner hitboxes determine lethal
// side contacts; the full player rectangle determines hazard/orb contacts.
pub const CUBE_GROUND_HITBOX_SCALE: f64 = 0.3;
pub const SHIP_GROUND_HITBOX_SCALE: f64 = 0.3;
pub const WAVE_GROUND_HITBOX_SCALE: f64 = 0.3;
pub const WAVE_DANGER_HITBOX_SIZE: f64 = 10. * GD_SCALE;
pub const SPIDER_SWEEP_SIZE: f64 = 8. * GD_SCALE;
pub const SURFACE_CONTACT_TOLERANCE: f64 = 1.;
pub const PLATFORM_HEIGHT: f64 = 32.;
pub const MINISPIKE_HEIGHT: f64 = 40.;
pub const JUMP_PAD_WIDTH: f64 = 70.;
pub const PORTAL_WIDTH: f64 = 90.;
pub const PORTAL_HEIGHT: f64 = 240.;
pub const TRIGGER_WIDTH: f64 = 40.;
pub const SPIKE_HITBOX_WIDTH: f64 = 16.;
pub const SPIKE_HITBOX_HEIGHT: f64 = 32.;
pub const SPECIAL_OBJECT_HITBOX_WIDTH: f64 = 16.;
pub const SPECIAL_OBJECT_HITBOX_HEIGHT: f64 = 20.;
pub const SPECIAL_OBJECT_HITBOX_OFFSET_Y: f64 = -36.;
pub const MINISPIKE_HITBOX_SIZE: f64 = 10.;
pub const MINISPIKE_HITBOX_OFFSET_Y: f64 = -13.;
pub const JUMP_PAD_HITBOX_HEIGHT: f64 = 10.;
pub const JUMP_PAD_HITBOX_OFFSET_Y: f64 = -35.;
pub const END_LEVEL_INDENT: f64 = 500.;
// Broad-phase search margins must cover nearby collider widths after rotation.
pub const COLLISION_SEARCH_BEHIND: f64 = 400.;
pub const COLLISION_SEARCH_AHEAD: f64 = 120.;

// Camera and floor transition. C++ used milliseconds for the transition duration.
pub const CAMERA_FOLLOW_SPEED: f64 = 1.25;
pub const MAX_SHIFT_Y: f64 = 400.;
pub const PLAYER_TO_GROUND_PROXIMITY_THRESHOLD: f64 = 320.;
pub const GROUND_HEIGHT_FOR_CUBE: f64 = 420.;
pub const GROUND_HEIGHT_FOR_WAVE_AND_SHIP: f64 = 260.;
pub const GROUND_TRANSITION_TIME: f64 = 0.4;
pub const BACKGROUND_SPEED: f64 = 50.;
pub const MENU_GROUND_SPEED: f64 = 300.;
pub const MENU_JUMP_PAUSE: f64 = 0.18;

// Presentation timing is independent of the physics tick rate.
pub const RESPAWN_DELAY: f64 = 1.0;
pub const SPAWN_REVEAL_TIME: f64 = 0.30;
pub const RESTART_FADE_TIME: f64 = 0.16;
pub const DEATH_FADE_TIME: f64 = 0.2;
pub const TRANSITION_MAX_FRAME_DELTA: f64 = 0.05;

/// Vanilla normal-size ship: stronger reversal, softer acceleration near falling.
pub fn ship_acceleration(gravity: f64, velocity: f64, held: bool) -> f64 {
    let falling = -gravity * velocity < 2. * 0.958199 * GD_VELOCITY_UNIT;
    let factor = match (held, falling) {
        (true, true) => -0.5,
        (true, false) => -0.4,
        (false, true) => 0.32,
        (false, false) => 0.48,
    };
    gravity * SHIP_GRAVITY * factor
}

/// GD 2.2 rounds native Y velocity to three decimal places on every change.
pub fn quantize_velocity(velocity: f64) -> f64 {
    (velocity / GD_VELOCITY_UNIT * 1000.).round() / 1000. * GD_VELOCITY_UNIT
}
