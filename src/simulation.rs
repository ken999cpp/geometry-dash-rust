use crate::{
    constants::{self, ship_acceleration},
    geometry::{Point, Rect},
    level::{GROUND, Level},
};

pub use crate::constants::{CEILING, DT, GRAVITY, HZ, JUMP, ROBOT_HOLD, SPEED};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlayerMode {
    #[default]
    Cube,
    Ship,
    Wave,
    Robot,
    Spider,
    Ball,
}
impl PlayerMode {
    pub const ALL: [Self; 6] = [
        Self::Cube,
        Self::Ship,
        Self::Wave,
        Self::Robot,
        Self::Spider,
        Self::Ball,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Cube => "cube",
            Self::Ship => "ship",
            Self::Wave => "wave",
            Self::Robot => "robot",
            Self::Spider => "spider",
            Self::Ball => "ball",
        }
    }
    pub fn size(self) -> Point {
        if self == Self::Wave {
            Point::new(constants::WAVE_WIDTH, constants::WAVE_HEIGHT)
        } else if self == Self::Ship {
            Point::new(constants::SHIP_SIZE, constants::SHIP_SIZE)
        } else if self == Self::Spider {
            Point::new(27. * constants::GD_SCALE, 27. * constants::GD_SCALE)
        } else {
            Point::new(constants::CUBE_SIZE, constants::CUBE_SIZE)
        }
    }
    pub fn from_portal(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|m| s == format!("{}_portal", m.name()))
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TickInput {
    pub held: bool,
    pub pressed: bool,
    pub released: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub position: Point,
    pub previous: Point,
    pub velocity: Point,
    pub mode: PlayerMode,
    pub gravity: f64,
    pub grounded: bool,
    pub rotation: f64,
    pub robot_hold: f64,
    /// A new press can wait for one orb until another jump consumes it.
    pub orb_armed: bool,
    pub input_held: bool,
    pub robot_boost_invalidated: bool,
    pub last_surface: Option<usize>,
    pub snap_distance: f64,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            position: Point::new(
                constants::CUBE_SIZE / 2.,
                GROUND + constants::CUBE_SIZE / 2.,
            ),
            previous: Point::new(
                constants::CUBE_SIZE / 2.,
                GROUND + constants::CUBE_SIZE / 2.,
            ),
            velocity: Point::new(SPEED, 0.),
            mode: PlayerMode::Cube,
            gravity: -1.,
            grounded: true,
            rotation: 0.,
            robot_hold: 0.,
            orb_armed: false,
            input_held: false,
            robot_boost_invalidated: true,
            last_surface: None,
            snap_distance: 0.,
        }
    }
}
impl Player {
    pub fn rect(&self) -> Rect {
        Rect::new(self.position, self.mode.size(), 0.)
    }
    pub fn danger(&self) -> Rect {
        if self.mode == PlayerMode::Wave {
            Rect::new(
                self.position,
                Point::new(
                    constants::WAVE_DANGER_HITBOX_SIZE,
                    constants::WAVE_DANGER_HITBOX_SIZE,
                ),
                0.,
            )
        } else {
            self.rect()
        }
    }
    pub fn ground_hitbox(&self) -> Rect {
        Rect::new(
            self.position,
            self.mode.size()
                * match self.mode {
                    PlayerMode::Wave => constants::WAVE_GROUND_HITBOX_SCALE,
                    PlayerMode::Ship => constants::SHIP_GROUND_HITBOX_SCALE,
                    _ => constants::CUBE_GROUND_HITBOX_SCALE,
                },
            0.,
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Status {
    Running,
    Dead(f64),
    Complete,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameEvent {
    Died,
    Restarted(f64),
    Completed,
    ModeChanged(PlayerMode),
    Orb,
    Pad,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorTween {
    pub current: [f32; 4],
    start: [f32; 4],
    target: [f32; 4],
    elapsed: f64,
    duration: f64,
}
impl ColorTween {
    fn new(color: [f32; 4]) -> Self {
        Self {
            current: color,
            start: color,
            target: color,
            elapsed: 0.,
            duration: 0.,
        }
    }
    fn set(&mut self, target: [f32; 4], duration: f64) {
        self.start = self.current;
        self.target = target;
        self.elapsed = 0.;
        self.duration = duration;
        if duration <= 0. {
            self.current = target;
        }
    }
    fn tick(&mut self) {
        self.advance(DT);
    }
    fn advance(&mut self, seconds: f64) {
        self.elapsed += seconds;
        let t = if self.duration <= 0. {
            1.
        } else {
            (self.elapsed / self.duration).clamp(0., 1.)
        } as f32;
        let t = if t < 0.5 {
            2. * t * t
        } else {
            1. - (-2. * t + 2.).powi(2) / 2.
        };
        for i in 0..4 {
            self.current[i] = self.start[i] + (self.target[i] - self.start[i]) * t;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub player: Player,
    pub active: Vec<bool>,
    pub background: ColorTween,
    pub ground: ColorTween,
    pub elapsed: f64,
    pub floor: f64,
    pub corridor_floor: f64,
    pub ceiling: f64,
    pub floor_speed: f64,
    pub camera_y: f64,
    pub tick: u64,
}
#[derive(Clone, Copy, Debug)]
enum Kind {
    Start,
    Solid,
    Hazard,
    Pad,
    Orb,
    Portal(PlayerMode),
    Gravity(f64),
    Trigger(u8, [f32; 4], f64),
    End,
}
#[derive(Clone, Copy, Debug)]
struct Collider {
    id: usize,
    rect: Rect,
    kind: Kind,
}

pub struct Simulation {
    pub level: Level,
    pub state: Snapshot,
    pub status: Status,
    pub practice: bool,
    pub checkpoints: Vec<Snapshot>,
    pub events: Vec<GameEvent>,
    /// Physics vertices, including direction changes between render frames.
    pub wave_trail: VecDeque<Point>,
    colliders: Vec<Collider>,
    length: f64,
    start_state: Snapshot,
}
impl Simulation {
    pub fn new(level: Level) -> Result<Self, String> {
        level.validate_playable()?;
        let colliders = level
            .objects
            .iter()
            .enumerate()
            .map(|(id, o)| {
                let kind = match o.kind.as_str() {
                    "block" | "platform" => Kind::Solid,
                    "spike" | "minispike" | "special_object" => Kind::Hazard,
                    "jump_pad" => Kind::Pad,
                    "yellow_orb" => Kind::Orb,
                    "end_level" => Kind::End,
                    "start_pos" => Kind::Start,
                    "ReverseGravityPortal" => Kind::Gravity(1.),
                    "NormalGravityPortal" => Kind::Gravity(-1.),
                    "trigger" => {
                        let channel = match o.trigger_type.as_deref() {
                            Some("ground_color_change") => 1,
                            Some("background_color_change") => 0,
                            _ => 2,
                        };
                        let color = parse_color(
                            o.trigger_params
                                .get("color")
                                .and_then(|v| v.as_str())
                                .unwrap_or("#ffffff"),
                            o.trigger_params
                                .get("alpha")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(1.) as f32,
                        );
                        let duration = o
                            .trigger_params
                            .get("duration")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.)
                            .max(0.);
                        Kind::Trigger(channel, color, duration)
                    }
                    k => Kind::Portal(PlayerMode::from_portal(k).expect("validated portal")),
                };
                Collider {
                    id,
                    rect: o.hitbox(),
                    kind,
                }
            })
            .collect::<Vec<_>>();
        let length = level.length();
        let state = Self::initial(level.objects.len());
        let mut sim = Self {
            level,
            start_state: state.clone(),
            state,
            status: Status::Running,
            practice: false,
            checkpoints: Vec::new(),
            events: Vec::with_capacity(8),
            wave_trail: VecDeque::with_capacity(512),
            colliders,
            length,
        };
        // Sort once; preserve JSON IDs to break ties deterministically.
        sim.colliders.sort_by(|a, b| {
            a.rect
                .bounds()
                .0
                .x
                .total_cmp(&b.rect.bounds().0.x)
                .then(a.id.cmp(&b.id))
        });
        sim.apply_start_triggers();
        sim.start_state = sim.state.clone();
        Ok(sim)
    }
    fn initial(count: usize) -> Snapshot {
        Snapshot {
            player: Player::default(),
            active: vec![false; count],
            background: ColorTween::new([0.12, 0.2, 0.8, 1.]),
            ground: ColorTween::new([0.03, 0.08, 0.4, 1.]),
            elapsed: 0.,
            floor: GROUND,
            corridor_floor: constants::GROUND_HEIGHT_FOR_WAVE_AND_SHIP,
            ceiling: CEILING,
            floor_speed: 400.,
            camera_y: 0.,
            tick: 0,
        }
    }
    fn apply_start_triggers(&mut self) {
        for i in 0..self.colliders.len() {
            let c = self.colliders[i];
            if c.rect.center.x <= self.state.player.position.x
                && matches!(c.kind, Kind::Trigger(..))
            {
                self.activate(c);
            }
        }
    }
    pub fn progress(&self) -> u32 {
        ((self.state.player.position.x / self.length * 100.).clamp(0., 100.)) as u32
    }
    pub fn checkpoint(&mut self) {
        if self.practice && self.status == Status::Running {
            self.checkpoints.push(self.state.clone());
        }
    }
    pub fn remove_checkpoint(&mut self) {
        self.checkpoints.pop();
    }
    pub fn restart(&mut self, from_checkpoint: bool) {
        if from_checkpoint
            && self.practice
            && let Some(cp) = self.checkpoints.last()
        {
            self.state.clone_from(cp);
        } else {
            self.state = self.start_state.clone();
        }
        self.state.player.previous = self.state.player.position;
        self.status = Status::Running;
        self.wave_trail.clear();
        self.events.push(GameEvent::Restarted(self.state.elapsed));
    }
    fn kill(&mut self) {
        if self.status == Status::Running {
            self.status = Status::Dead(constants::RESPAWN_DELAY);
            self.events.push(GameEvent::Died);
        }
    }
    fn complete(&mut self) {
        if self.status == Status::Running {
            self.status = Status::Complete;
            self.events.push(GameEvent::Completed);
        }
    }
    pub fn tick(&mut self, input: TickInput) {
        self.events.clear();
        match self.status {
            Status::Complete => return,
            Status::Dead(t) => {
                if t <= DT {
                    self.restart(true);
                } else {
                    self.status = Status::Dead(t - DT);
                }
                return;
            }
            Status::Running => {}
        }
        self.state.tick += 1;
        self.state.elapsed += DT;
        self.state.background.tick();
        self.state.ground.tick();
        let flight = matches!(
            self.state.player.mode,
            PlayerMode::Ship | PlayerMode::Wave | PlayerMode::Ball | PlayerMode::Spider
        );
        let floor_target = if flight {
            self.state.corridor_floor
        } else {
            GROUND
        };
        let floor_step = self.state.floor_speed * DT;
        self.state.floor += (floor_target - self.state.floor).clamp(-floor_step, floor_step);
        self.state.player.previous = self.state.player.position;
        let player = &mut self.state.player;
        let input = TickInput {
            pressed: input.pressed || (input.held && !player.input_held),
            ..input
        };
        if input.pressed || (input.held && !player.input_held) {
            player.orb_armed = true;
        } else if !input.held {
            player.orb_armed = false;
        }
        player.input_held = input.held;
        // An orb already in reach takes the press on this tick. A ground jump
        // otherwise consumes it, preventing the same hold from using later orbs.
        let mut orb_request = player.orb_armed;
        self.move_player(input);
        self.record_wave_trail();
        let x = self.state.player.position.x;
        // Search margins include rotated source-object bounds.
        let start = self
            .colliders
            .partition_point(|c| c.rect.bounds().0.x < x - constants::COLLISION_SEARCH_BEHIND);
        let end = self
            .colliders
            .partition_point(|c| c.rect.bounds().0.x < x + constants::COLLISION_SEARCH_AHEAD);
        for i in start..end {
            let c = self.colliders[i];
            if matches!(c.kind, Kind::Solid) {
                self.collide_solid(c);
            }
            if self.status != Status::Running {
                return;
            }
        }
        for i in 0..self.colliders.len() {
            let c = self.colliders[i];
            if matches!(c.kind, Kind::Trigger(..) | Kind::End)
                && c.rect.center.x <= x
                && !self.state.active[c.id]
            {
                self.activate(c);
            }
        }
        for i in start..end {
            let c = self.colliders[i];
            let touching = if matches!(c.kind, Kind::Orb) {
                circle_touches_rect(c.rect.center, c.rect.half.x, self.state.player.rect())
            } else if matches!(c.kind, Kind::Hazard) {
                self.state.player.danger().overlaps(c.rect)
            } else {
                self.state.player.rect().overlaps(c.rect)
            };
            if !touching {
                continue;
            }
            match c.kind {
                Kind::Hazard => self.kill(),
                Kind::Orb if orb_request && !self.state.active[c.id] => {
                    orb_request = false;
                    self.state.player.orb_armed = false;
                    self.state.active[c.id] = true;
                    let factor = match self.state.player.mode {
                        PlayerMode::Ball | PlayerMode::Spider => 0.7,
                        PlayerMode::Robot => 0.9,
                        _ => 1.,
                    };
                    self.state.player.velocity.y = constants::quantize_velocity(
                        -self.state.player.gravity * constants::ORB_JUMP_FORCE * factor,
                    );
                    self.state.player.robot_boost_invalidated = true;
                    self.state.player.grounded = false;
                    self.events.push(GameEvent::Orb);
                }
                Kind::Pad
                    if self.state.player.mode != PlayerMode::Wave && !self.state.active[c.id] =>
                {
                    orb_request = false;
                    self.state.player.orb_armed = false;
                    self.state.active[c.id] = true;
                    let factor = if matches!(
                        self.state.player.mode,
                        PlayerMode::Ball | PlayerMode::Spider
                    ) {
                        0.6
                    } else {
                        1.
                    };
                    self.state.player.velocity.y = constants::quantize_velocity(
                        -self.state.player.gravity * constants::JUMP_PAD_CUBE_FORCE * factor,
                    );
                    self.state.player.robot_boost_invalidated = true;
                    self.state.player.grounded = false;
                    self.events.push(GameEvent::Pad);
                }
                Kind::Portal(_) | Kind::Gravity(_) if !self.state.active[c.id] => self.activate(c),
                _ => {}
            }
            if self.status != Status::Running {
                return;
            }
        }
        let p = &self.state.player;
        let target = if flight {
            (self.state.floor + self.state.ceiling) / 2. - constants::GAME_CAMERA_Y
        } else {
            (p.position.y - GROUND - constants::PLAYER_TO_GROUND_PROXIMITY_THRESHOLD)
                .clamp(0., constants::MAX_SHIFT_Y)
        };
        self.state.camera_y +=
            (target - self.state.camera_y) * (1. - (-constants::CAMERA_FOLLOW_SPEED * DT).exp());
        if x >= self.length {
            self.complete();
        }
    }
    fn record_wave_trail(&mut self) {
        let p = &self.state.player;
        if p.mode != PlayerMode::Wave {
            self.wave_trail.clear();
            return;
        }
        if self
            .wave_trail
            .back()
            .is_none_or(|last| *last != p.previous)
        {
            self.wave_trail.clear();
            self.wave_trail.push_back(p.previous);
        }
        let previous = p.previous;
        let current = p.position;
        let dx = current.x - previous.x;
        let dy = (current.y - previous.y).abs();
        // A boundary contact can occur between ticks. Keep the 45-degree
        // segment up to that exact contact, followed by the horizontal slide.
        if p.velocity.y == 0. && dy > 1e-7 && dy < dx - 1e-7 {
            self.append_wave_point(Point::new(previous.x + dy, current.y));
        }
        self.append_wave_point(current);
        while self.wave_trail.len() > 2048
            || (self.wave_trail.len() > 2
                && self.wave_trail[1].x < current.x - constants::VIEWPORT_HEIGHT * 4.)
        {
            self.wave_trail.pop_front();
        }
    }
    fn append_wave_point(&mut self, current: Point) {
        if self.wave_trail.len() >= 2 {
            let a = self.wave_trail[self.wave_trail.len() - 2];
            let b = *self.wave_trail.back().unwrap();
            let ab = b - a;
            let bc = current - b;
            if (ab.x * bc.y - ab.y * bc.x).abs() < 1e-6 && ab.dot(bc) > 0. {
                self.wave_trail.pop_back();
            }
        }
        self.wave_trail.push_back(current);
    }
    fn move_player(&mut self, input: TickInput) {
        if self.state.player.mode == PlayerMode::Spider
            && self.state.player.orb_armed
            && (input.held || input.pressed)
            && self.state.player.grounded
        {
            self.state.player.orb_armed = false;
            self.spider_switch();
        }
        let p = &mut self.state.player;
        let mut jumped = false;
        match p.mode {
            PlayerMode::Cube | PlayerMode::Robot => {
                let jumping = p.grounded
                    && (if p.mode == PlayerMode::Cube {
                        input.held || input.pressed
                    } else {
                        p.orb_armed && (input.held || input.pressed)
                    });
                if jumping {
                    jumped = true;
                    p.orb_armed = false;
                    let impulse = if p.mode == PlayerMode::Robot {
                        constants::ROBOT_JUMP
                    } else {
                        JUMP
                    };
                    p.velocity.y = -p.gravity * impulse;
                    p.grounded = false;
                    p.robot_hold = 0.;
                    p.robot_boost_invalidated = false;
                }
                let holding = p.mode == PlayerMode::Robot
                    && input.held
                    && !p.robot_boost_invalidated
                    && p.robot_hold < ROBOT_HOLD
                    && p.velocity.y * p.gravity < 0.;
                // Fresh presses set the jump before the physics update, so
                // gravity applies on takeoff. Automatic held jumps after a
                // landing start within the update and keep one extra gravity step.
                if !jumping || input.pressed {
                    let gravity = if p.mode == PlayerMode::Robot {
                        constants::ROBOT_GRAVITY
                    } else {
                        GRAVITY
                    };
                    p.velocity.y += p.gravity
                        * gravity
                        * DT
                        * if holding {
                            constants::ROBOT_HOLD_GRAVITY_FACTOR
                        } else {
                            1.
                        };
                }
                if holding && !jumping {
                    p.robot_hold += DT;
                }
                if !input.held && !input.pressed {
                    p.robot_boost_invalidated = true;
                }
                let (rotation_speed, rotation_lerp) = if p.mode == PlayerMode::Robot {
                    (
                        constants::ROBOT_ROTATION_SPEED,
                        constants::ROBOT_ROTATION_LERP_SPEED,
                    )
                } else {
                    (
                        constants::CUBE_ROTATION_SPEED,
                        constants::CUBE_ROTATION_LERP_SPEED,
                    )
                };
                if !p.grounded {
                    p.rotation += p.gravity * rotation_speed * DT;
                } else {
                    let step = constants::ROTATION_ALIGNMENT_STEP;
                    p.rotation +=
                        ((p.rotation / step).round() * step - p.rotation) * rotation_lerp * DT;
                }
            }
            PlayerMode::Ship => {
                p.velocity.y +=
                    ship_acceleration(p.gravity, p.velocity.y, input.held || input.pressed) * DT;
                let rising = (-p.gravity * p.velocity.y).clamp(
                    -constants::SHIP_MAX_FALL_SPEED,
                    constants::SHIP_MAX_RISE_SPEED,
                );
                p.velocity.y = -p.gravity * rising;
            }
            PlayerMode::Wave => {
                p.velocity.y = constants::WAVE_SPEED
                    * if input.held || input.pressed {
                        -p.gravity
                    } else {
                        p.gravity
                    };
            }
            PlayerMode::Ball => {
                if p.orb_armed && (input.held || input.pressed) && p.grounded {
                    p.orb_armed = false;
                    p.gravity = -p.gravity;
                    // flipGravity halves the jump before the ball's 0.6 modifier.
                    p.velocity.y = p.gravity * JUMP * 0.3;
                    p.grounded = false;
                    jumped = true;
                }
                if !jumped {
                    p.velocity.y += p.gravity * constants::BALL_GRAVITY * DT;
                }
                p.rotation += p.gravity * constants::BALL_ROTATION_SPEED * DT;
            }
            PlayerMode::Spider => {
                p.velocity.y += p.gravity * constants::SPIDER_GRAVITY * DT;
            }
        }
        if !matches!(p.mode, PlayerMode::Ship | PlayerMode::Wave) && !jumped {
            p.velocity.y = if p.gravity < 0. {
                p.velocity.y.max(-constants::MAX_FALL_SPEED)
            } else {
                p.velocity.y.min(constants::MAX_FALL_SPEED)
            };
        }
        if p.mode != PlayerMode::Wave {
            p.velocity.y = constants::quantize_velocity(p.velocity.y);
        }
        p.position.x += SPEED * DT;
        p.position.y += p.velocity.y * DT;
        p.grounded = false;
        let half = if p.mode == PlayerMode::Wave {
            10. * constants::GD_SCALE
        } else {
            p.mode.size().y / 2.
        };
        if p.position.y - half <= self.state.floor {
            p.position.y = self.state.floor + half;
            p.velocity.y = p.velocity.y.max(0.);
            p.grounded = p.gravity < 0.;
        }
        if p.position.y + half >= self.state.ceiling
            && (matches!(
                p.mode,
                PlayerMode::Ship | PlayerMode::Wave | PlayerMode::Spider | PlayerMode::Ball
            ) || p.gravity > 0.)
        {
            p.position.y = self.state.ceiling - half;
            p.velocity.y = p.velocity.y.min(0.);
            p.grounded = p.gravity > 0.;
        }
        if p.mode == PlayerMode::Ship {
            let movement = p.position - p.previous;
            let angle = movement
                .y
                .atan2(movement.x)
                .to_degrees()
                .clamp(-constants::SHIP_MAX_ANGLE, constants::SHIP_MAX_ANGLE);
            p.rotation += (angle - p.rotation) * constants::SHIP_ROTATION_LERP * DT;
        } else if p.mode == PlayerMode::Wave {
            let angle = if p.velocity.y == 0. {
                0.
            } else {
                p.velocity.y.signum() * constants::WAVE_ANGLE
            };
            p.rotation += (angle - p.rotation) * constants::WAVE_ROTATION_LERP * DT;
        }
    }
    fn spider_switch(&mut self) {
        let p = &self.state.player;
        let half = p.mode.size().y / 2.;
        let mut target = if p.gravity < 0. {
            self.state.ceiling - half
        } else {
            self.state.floor + half
        };
        for c in &self.colliders {
            if !matches!(c.kind, Kind::Solid) {
                continue;
            }
            let (lo, hi) = c.rect.bounds();
            if hi.x <= p.position.x
                || lo.x >= p.position.x + p.mode.size().x / 2. + constants::GD_SCALE
            {
                continue;
            }
            if let Some((bottom, top)) =
                vertical_span(c.rect, p.position.x.clamp(lo.x + 1e-6, hi.x - 1e-6))
            {
                if p.gravity < 0. && bottom - half > p.position.y {
                    target = target.min(bottom - half);
                }
                if p.gravity > 0. && top + half < p.position.y {
                    target = target.max(top + half);
                }
            }
        }
        let sweep = Rect::new(
            Point::new(p.position.x, (p.position.y + target) / 2.),
            Point::new(
                constants::SPIDER_SWEEP_SIZE,
                (target - p.position.y).abs() + constants::SPIDER_SWEEP_SIZE,
            ),
            0.,
        );
        let nearest_hazard = self
            .colliders
            .iter()
            .filter(|c| matches!(c.kind, Kind::Hazard) && sweep.overlaps(c.rect))
            .filter(|c| (c.rect.center.y - p.position.y) * -p.gravity > 0.)
            .filter(|c| (c.rect.center.y - p.position.y).abs() < (target - p.position.y).abs())
            .min_by(|a, b| {
                ((a.rect.center.y - p.position.y).abs())
                    .total_cmp(&(b.rect.center.y - p.position.y).abs())
            });
        if let Some(hazard) = nearest_hazard {
            target = hazard.rect.center.y;
        }
        self.state.player.position.y = target;
        self.state.player.previous.y = target;
        self.state.player.gravity = -self.state.player.gravity;
        self.state.player.velocity.y = self.state.player.gravity * constants::GD_VELOCITY_UNIT;
        self.state.player.robot_boost_invalidated = true;
        if nearest_hazard.is_some() {
            self.kill();
        }
    }
    fn collide_solid(&mut self, collider: Collider) {
        let rect = collider.rect;
        let p = &mut self.state.player;
        if !p.rect().overlaps(rect) {
            return;
        }
        if p.mode == PlayerMode::Wave {
            if p.ground_hitbox().overlaps(rect) {
                self.kill();
            }
            return;
        }
        // A leading/trailing edge can support the player before its center is
        // above the platform. Preserve the actual surface for rotated blocks.
        let (lo, hi) = rect.bounds();
        let sample_x = p.position.x.clamp(lo.x + 1e-6, hi.x - 1e-6);
        let Some((bottom, top)) = vertical_span(rect, sample_x) else {
            if p.ground_hitbox().overlaps(rect) {
                self.kill();
            }
            return;
        };
        let half = p.mode.size().y / 2.;
        let prev_bottom = p.previous.y - half;
        let prev_top = p.previous.y + half;
        let mut landed = false;
        let snap = if p.mode == PlayerMode::Ship { 6. } else { 10. } * constants::GD_SCALE;
        if p.velocity.y <= 0. && prev_bottom >= top - snap {
            p.position.y = top + half;
            p.velocity.y = 0.;
            p.grounded = p.gravity < 0.;
            landed = p.grounded;
        } else if p.velocity.y >= 0.
            && prev_top <= bottom + snap
            && (p.gravity > 0. || matches!(p.mode, PlayerMode::Ship | PlayerMode::Ball))
        {
            p.position.y = bottom - half;
            p.velocity.y = 0.;
            p.grounded = p.gravity > 0.;
            landed = p.grounded;
        } else if p.ground_hitbox().overlaps(rect) {
            self.kill();
        }
        if landed && self.state.player.mode == PlayerMode::Cube {
            self.snap_to_stair(collider);
        }
    }
    fn snap_to_stair(&mut self, current: Collider) {
        let p = &mut self.state.player;
        if let Some(previous) = p
            .last_surface
            .and_then(|id| self.colliders.iter().find(|c| c.id == id))
            .filter(|c| c.id != current.id)
        {
            let delta = current.rect.center - previous.rect.center;
            let tolerance = constants::GD_SCALE;
            let step = constants::BLOCK_SIZE;
            let up = -p.gravity;
            if [
                (4. * step, up * step),
                (5. * step, -up * step),
                (3. * step, up * 2. * step),
            ]
            .iter()
            .any(|&(x, y)| (delta.x - x).abs() <= tolerance && (delta.y - y).abs() <= tolerance)
            {
                let desired = current.rect.center.x + p.snap_distance;
                p.position.x += (desired - p.position.x).clamp(-tolerance, tolerance);
            }
        }
        p.last_surface = Some(current.id);
        p.snap_distance = p.position.x - current.rect.center.x;
    }
    fn activate(&mut self, c: Collider) {
        self.state.active[c.id] = true;
        match c.kind {
            Kind::Portal(mode) => {
                self.configure_corridor(mode, c.rect.center.y);
                let old_mode = self.state.player.mode;
                if old_mode == mode {
                    return;
                }
                // Flying mode toggles halve momentum both on entry and exit.
                if matches!(old_mode, PlayerMode::Ship | PlayerMode::Wave) {
                    self.state.player.velocity.y *= 0.5;
                }
                if matches!(mode, PlayerMode::Ship | PlayerMode::Wave) {
                    self.state.player.velocity.y *= 0.5;
                }
                self.state.player.mode = mode;
                self.state.player.last_surface = None;
                self.state.player.robot_hold = ROBOT_HOLD;
                self.state.player.robot_boost_invalidated = true;
                self.state.player.rotation = if self.state.player.gravity > 0.
                    && matches!(mode, PlayerMode::Cube | PlayerMode::Ball)
                {
                    180.
                } else {
                    0.
                };
                if matches!(mode, PlayerMode::Ship | PlayerMode::Wave) {
                    self.state.player.grounded = false;
                }
                self.wave_trail.clear();
                self.events.push(GameEvent::ModeChanged(mode));
            }
            Kind::Gravity(g) if self.state.player.gravity != g => {
                self.state.player.gravity = g;
                self.state.player.velocity.y *= 0.5;
                self.state.player.grounded = false;
            }
            Kind::Trigger(channel, color, duration) => {
                if channel != 1 {
                    self.state.background.set(color, duration);
                }
                if channel != 0 {
                    self.state.ground.set(color, duration);
                }
            }
            Kind::End => self.complete(),
            _ => {}
        }
    }
    fn configure_corridor(&mut self, mode: PlayerMode, center_y: f64) {
        if matches!(
            mode,
            PlayerMode::Ship | PlayerMode::Wave | PlayerMode::Ball | PlayerMode::Spider
        ) {
            let height = constants::BLOCK_SIZE
                * if matches!(mode, PlayerMode::Ship | PlayerMode::Wave) {
                    10.
                } else {
                    8.
                };
            let center = GROUND
                + ((center_y - GROUND) / constants::BLOCK_SIZE).floor() * constants::BLOCK_SIZE;
            self.state.corridor_floor = (center - height / 2.).max(GROUND);
            self.state.ceiling = self.state.corridor_floor + height;
            self.state.floor_speed = (self.state.corridor_floor - self.state.floor).abs()
                / constants::GROUND_TRANSITION_TIME;
        } else {
            self.state.floor_speed =
                (GROUND - self.state.floor).abs() / constants::GROUND_TRANSITION_TIME;
        }
    }

    /// Editor playtest only. Normal play always starts at the level beginning.
    pub fn start_at(&mut self, marker: usize) -> Result<(), String> {
        let object = self
            .level
            .objects
            .get(marker)
            .filter(|o| o.kind == "start_pos")
            .ok_or("Select a start position")?;
        let position = object.rect().center;
        let mode_name = object
            .extra
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("auto")
            .to_owned();
        let gravity_name = object
            .extra
            .get("gravity")
            .and_then(|v| v.as_str())
            .unwrap_or("auto")
            .to_owned();
        let elapsed = ((position.x - constants::CUBE_SIZE / 2.) / SPEED).max(0.);
        self.state = Self::initial(self.level.objects.len());
        let mut cursor = 0.;
        let mut mode = PlayerMode::Cube;
        let mut gravity = -1.;
        for i in 0..self.colliders.len() {
            let c = self.colliders[i];
            if c.rect.center.x > position.x {
                continue;
            }
            self.state.active[c.id] = true;
            match c.kind {
                Kind::Trigger(..) => {
                    let at = ((c.rect.center.x - constants::CUBE_SIZE / 2.) / SPEED)
                        .max(cursor)
                        .min(elapsed);
                    self.state.background.advance(at - cursor);
                    self.state.ground.advance(at - cursor);
                    self.activate(c);
                    cursor = at;
                }
                Kind::Portal(m) => {
                    mode = m;
                    self.configure_corridor(m, c.rect.center.y);
                }
                Kind::Gravity(g) => gravity = g,
                _ => {}
            }
        }
        self.state.background.advance(elapsed - cursor);
        self.state.ground.advance(elapsed - cursor);
        if mode_name != "auto" {
            mode = PlayerMode::ALL
                .into_iter()
                .find(|m| m.name() == mode_name)
                .ok_or("Unknown start mode")?;
            self.configure_corridor(mode, position.y);
        }
        gravity = match gravity_name.as_str() {
            "normal" => -1.,
            "reverse" => 1.,
            _ => gravity,
        };
        let bounded = matches!(
            mode,
            PlayerMode::Ship | PlayerMode::Wave | PlayerMode::Ball | PlayerMode::Spider
        );
        self.state.floor = if bounded {
            self.state.corridor_floor
        } else {
            GROUND
        };
        self.state.elapsed = elapsed;
        self.state.tick = (elapsed / DT).round() as u64;
        let p = &mut self.state.player;
        p.mode = mode;
        p.gravity = gravity;
        p.position = position;
        p.previous = position;
        p.grounded = false;
        p.robot_boost_invalidated = true;
        let half = mode.size().y / 2.;
        if p.position.y - half <= self.state.floor && gravity < 0. {
            p.position.y = self.state.floor + half;
            p.grounded = true;
        }
        if p.position.y + half >= self.state.ceiling && gravity > 0. {
            p.position.y = self.state.ceiling - half;
            p.grounded = true;
        }
        p.previous = p.position;
        self.state.camera_y = if bounded {
            (self.state.floor + self.state.ceiling) / 2. - constants::GAME_CAMERA_Y
        } else {
            (p.position.y - GROUND - constants::PLAYER_TO_GROUND_PROXIMITY_THRESHOLD)
                .clamp(0., constants::MAX_SHIFT_Y)
        };
        self.status = Status::Running;
        self.wave_trail.clear();
        self.checkpoints.clear();
        self.events.clear();
        self.start_state = self.state.clone();
        Ok(())
    }
}

/// Intersection of a rotated rectangle with a vertical line; used for surface landing.
pub fn vertical_span(rect: Rect, x: f64) -> Option<(f64, f64)> {
    let [a, b] = rect.axes();
    let mut vertices = [Point::default(); 4];
    for (i, (sx, sy)) in [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)]
        .into_iter()
        .enumerate()
    {
        vertices[i] = rect.center + a * (rect.half.x * sx) + b * (rect.half.y * sy);
    }
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for i in 0..4 {
        let p = vertices[i];
        let q = vertices[(i + 1) % 4];
        if x < p.x.min(q.x) - 1e-7 || x > p.x.max(q.x) + 1e-7 {
            continue;
        }
        if (p.x - q.x).abs() < 1e-7 {
            lo = lo.min(p.y.min(q.y));
            hi = hi.max(p.y.max(q.y));
        } else {
            let y = p.y + (q.y - p.y) * (x - p.x) / (q.x - p.x);
            lo = lo.min(y);
            hi = hi.max(y);
        }
    }
    (lo <= hi).then_some((lo, hi))
}
pub fn parse_color(hex: &str, alpha: f32) -> [f32; 4] {
    let value = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0xffffff);
    [
        ((value >> 16) & 255) as f32 / 255.,
        ((value >> 8) & 255) as f32 / 255.,
        (value & 255) as f32 / 255.,
        alpha.clamp(0., 1.),
    ]
}

fn circle_touches_rect(center: Point, radius: f64, rect: Rect) -> bool {
    let [a, b] = rect.axes();
    let delta = center - rect.center;
    let dx = (delta.dot(a).abs() - rect.half.x).max(0.);
    let dy = (delta.dot(b).abs() - rect.half.y).max(0.);
    dx * dx + dy * dy <= radius * radius
}
