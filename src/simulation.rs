use crate::{
    constants::{self, ship_acceleration},
    geometry::{Point, Rect},
    level::{GROUND, Level},
};

pub use crate::constants::{CEILING, DT, GRAVITY, HZ, JUMP, ROBOT_HOLD, SPEED};

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
        self.elapsed += DT;
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
    pub camera_y: f64,
    pub tick: u64,
}
#[derive(Clone, Copy, Debug)]
enum Kind {
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
    colliders: Vec<Collider>,
    length: f64,
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
            state,
            status: Status::Running,
            practice: false,
            checkpoints: Vec::new(),
            events: Vec::with_capacity(8),
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
            self.state = Self::initial(self.level.objects.len());
            self.apply_start_triggers();
        }
        self.state.player.previous = self.state.player.position;
        self.status = Status::Running;
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
        let flight = matches!(self.state.player.mode, PlayerMode::Ship | PlayerMode::Wave);
        let floor_target = if flight {
            constants::GROUND_HEIGHT_FOR_WAVE_AND_SHIP
        } else {
            GROUND
        };
        let floor_step = (GROUND - constants::GROUND_HEIGHT_FOR_WAVE_AND_SHIP).abs() * DT
            / constants::GROUND_TRANSITION_TIME;
        self.state.floor += (floor_target - self.state.floor).clamp(-floor_step, floor_step);
        self.state.player.previous = self.state.player.position;
        let player = &mut self.state.player;
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
                self.collide_solid(c.rect);
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
            if !self.state.player.danger().overlaps(c.rect) {
                continue;
            }
            match c.kind {
                Kind::Hazard => self.kill(),
                Kind::Orb if orb_request && !self.state.active[c.id] => {
                    orb_request = false;
                    self.state.player.orb_armed = false;
                    self.state.active[c.id] = true;
                    self.state.player.velocity.y =
                        -self.state.player.gravity * constants::ORB_JUMP_FORCE;
                    self.state.player.grounded = false;
                    self.events.push(GameEvent::Orb);
                }
                Kind::Pad
                    if self.state.player.mode != PlayerMode::Wave && !self.state.active[c.id] =>
                {
                    orb_request = false;
                    self.state.player.orb_armed = false;
                    self.state.active[c.id] = true;
                    self.state.player.velocity.y = -self.state.player.gravity
                        * if self.state.player.mode == PlayerMode::Ship {
                            constants::JUMP_PAD_SHIP_FORCE
                        } else {
                            constants::JUMP_PAD_CUBE_FORCE
                        };
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
            0.
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
    fn move_player(&mut self, input: TickInput) {
        if self.state.player.mode == PlayerMode::Spider
            && input.pressed
            && self.state.player.grounded
        {
            self.state.player.orb_armed = false;
            self.spider_switch();
        }
        let p = &mut self.state.player;
        match p.mode {
            PlayerMode::Cube | PlayerMode::Robot => {
                let jumping = p.grounded
                    && (if p.mode == PlayerMode::Cube {
                        input.held || input.pressed
                    } else {
                        input.pressed
                    });
                if jumping {
                    p.orb_armed = false;
                    let impulse = if p.mode == PlayerMode::Robot {
                        constants::ROBOT_JUMP
                    } else {
                        JUMP
                    };
                    p.velocity.y = -p.gravity * impulse;
                    p.grounded = false;
                    p.robot_hold = 0.;
                }
                let holding = p.mode == PlayerMode::Robot
                    && input.held
                    && p.robot_hold < ROBOT_HOLD
                    && p.velocity.y * p.gravity < 0.;
                // Cube::update applies gravity before integrating every tick.
                {
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
                p.robot_hold += DT;
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
                    p.rotation -= rotation_speed * DT;
                } else {
                    let step = constants::ROTATION_ALIGNMENT_STEP;
                    p.rotation +=
                        ((p.rotation / step).round() * step - p.rotation) * rotation_lerp * DT;
                }
            }
            PlayerMode::Ship => {
                p.velocity.y +=
                    ship_acceleration(p.gravity, p.grounded, input.held || input.pressed) * DT;
                let rising = (-p.gravity * p.velocity.y).clamp(
                    -constants::SHIP_MAX_FALL_SPEED,
                    constants::SHIP_MAX_RISE_SPEED,
                );
                p.velocity.y = -p.gravity * rising;
            }
            PlayerMode::Wave => {
                p.velocity.y = constants::WAVE_SPEED / 2_f64.sqrt()
                    * if input.held { -p.gravity } else { p.gravity };
                p.rotation = if p.velocity.y > 0. {
                    constants::WAVE_ANGLE
                } else {
                    -constants::WAVE_ANGLE
                };
            }
            PlayerMode::Ball => {
                if input.pressed && p.grounded {
                    p.orb_armed = false;
                    p.gravity = -p.gravity;
                    p.grounded = false;
                }
                p.velocity.y += p.gravity * constants::BALL_GRAVITY * DT;
                p.rotation -= p.gravity * constants::BALL_ROTATION_SPEED * DT;
            }
            PlayerMode::Spider => {
                p.velocity.y += p.gravity * constants::SPIDER_GRAVITY * DT;
            }
        }
        p.position.x += SPEED * DT;
        // Ship::update in the C++ project advances Y twice using the new velocity.
        // Preserve that free-flight trajectory, then resolve surfaces once below.
        p.position.y += p.velocity.y
            * DT
            * if p.mode == PlayerMode::Ship {
                constants::SHIP_DISPLACEMENT_MULTIPLIER
            } else {
                1.
            };
        p.grounded = false;
        let half = p.mode.size().y / 2.;
        if p.position.y - half <= self.state.floor {
            p.position.y = self.state.floor + half;
            p.velocity.y = p.velocity.y.max(0.);
            p.grounded = p.gravity < 0.;
        }
        if p.position.y + half >= CEILING
            && (matches!(
                p.mode,
                PlayerMode::Ship | PlayerMode::Wave | PlayerMode::Spider | PlayerMode::Ball
            ) || p.gravity > 0.)
        {
            p.position.y = CEILING - half;
            p.velocity.y = p.velocity.y.min(0.);
            p.grounded = p.gravity > 0.;
        }
        if p.mode == PlayerMode::Ship {
            let angle = (p.velocity.y * constants::SHIP_ROTATION_FACTOR)
                .clamp(-constants::SHIP_MAX_ANGLE, constants::SHIP_MAX_ANGLE);
            p.rotation += (angle - p.rotation) * constants::SHIP_ROTATION_LERP * DT;
        }
    }
    fn spider_switch(&mut self) {
        let p = &self.state.player;
        let half = p.mode.size().y / 2.;
        let mut target = if p.gravity < 0. {
            CEILING - half
        } else {
            self.state.floor + half
        };
        for c in &self.colliders {
            if !matches!(c.kind, Kind::Solid) {
                continue;
            }
            if let Some((bottom, top)) = vertical_span(c.rect, p.position.x) {
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
        let hit = self
            .colliders
            .iter()
            .any(|c| matches!(c.kind, Kind::Hazard) && sweep.overlaps(c.rect));
        self.state.player.position.y = target;
        self.state.player.previous.y = target;
        self.state.player.gravity = -self.state.player.gravity;
        self.state.player.velocity.y = 0.;
        if hit {
            self.kill();
        }
    }
    fn collide_solid(&mut self, rect: Rect) {
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
        let Some((bottom, top)) = vertical_span(rect, p.position.x) else {
            if p.ground_hitbox().overlaps(rect) {
                self.kill();
            }
            return;
        };
        let half = p.mode.size().y / 2.;
        let prev_bottom = p.previous.y - half;
        let prev_top = p.previous.y + half;
        if p.velocity.y <= 0. && prev_bottom >= top - constants::SURFACE_CONTACT_TOLERANCE {
            p.position.y = top + half;
            p.velocity.y = 0.;
            p.grounded = p.gravity < 0.;
        } else if p.velocity.y >= 0.
            && prev_top <= bottom + constants::SURFACE_CONTACT_TOLERANCE
            && (p.gravity > 0. || p.mode == PlayerMode::Ship)
        {
            p.position.y = bottom - half;
            p.velocity.y = 0.;
            p.grounded = p.gravity > 0.;
        } else if p.ground_hitbox().overlaps(rect) {
            self.kill();
        }
    }
    fn activate(&mut self, c: Collider) {
        self.state.active[c.id] = true;
        match c.kind {
            Kind::Portal(mode) => {
                self.state.player.mode = mode;
                self.state.player.robot_hold = ROBOT_HOLD;
                self.events.push(GameEvent::ModeChanged(mode));
            }
            Kind::Gravity(g) => {
                self.state.player.gravity = g;
                self.state.player.velocity.y = 0.;
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
