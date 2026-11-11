//! Frame-time presentation phases, independent of fixed-step catch-up batches.
use super::{Game, Screen};
use crate::constants;
use bevy::prelude::*;

pub(super) const SPAWN_TIME: f64 = constants::SPAWN_REVEAL_TIME;
const RESTART_FADE: f64 = constants::RESTART_FADE_TIME;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Phase {
    #[default]
    Playing,
    Dying,
    Restarting,
    Spawning,
}
#[derive(Default)]
pub(super) struct AttemptFlow {
    pub phase: Phase,
    pub elapsed: f64,
    entered: bool,
}
#[derive(PartialEq, Debug)]
enum Next {
    None,
    Reset,
    Ready,
}
impl AttemptFlow {
    pub fn enter(&mut self, phase: Phase) {
        self.phase = phase;
        self.elapsed = 0.;
        self.entered = true;
    }
    pub fn playing(&self) -> bool {
        self.phase == Phase::Playing
    }
    fn advance(&mut self, delta: f64) -> Next {
        if self.playing() {
            return Next::None;
        }
        // Do not charge the frame that performed a reset/asset upload to its
        // newly visible phase. A hitch must not skip the death animation.
        if self.entered {
            self.entered = false;
            return Next::None;
        }
        self.elapsed += delta.clamp(0., constants::TRANSITION_MAX_FRAME_DELTA);
        match self.phase {
            Phase::Dying if self.elapsed + 1e-9 >= crate::constants::RESPAWN_DELAY => {
                self.enter(Phase::Spawning);
                Next::Reset
            }
            Phase::Restarting if self.elapsed + 1e-9 >= RESTART_FADE => {
                self.enter(Phase::Spawning);
                Next::Reset
            }
            Phase::Spawning if self.elapsed + 1e-9 >= SPAWN_TIME => {
                self.enter(Phase::Playing);
                Next::Ready
            }
            _ => Next::None,
        }
    }
    pub fn fade(&self) -> f32 {
        let t = match self.phase {
            Phase::Dying => {
                (self.elapsed - (constants::RESPAWN_DELAY - constants::DEATH_FADE_TIME))
                    / constants::DEATH_FADE_TIME
            }
            Phase::Restarting => self.elapsed / RESTART_FADE,
            Phase::Spawning => 1. - self.elapsed / SPAWN_TIME,
            Phase::Playing => 0.,
        }
        .clamp(0., 1.);
        (t * t * (3. - 2. * t)) as f32
    }
}
pub(super) fn advance(mut game: ResMut<Game>, time: Res<Time<Real>>) {
    game.block_fixed_frame = false;
    if game.screen != Screen::Playing || game.exit_dialog.is_some() {
        return;
    }
    match game.attempt.advance(time.delta_secs_f64()) {
        Next::Reset => game.reset_attempt(),
        Next::Ready => {
            game.block_fixed_frame = true;
            game.input = Default::default();
        }
        Next::None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn death_and_reveal_cannot_be_skipped_by_a_hitch() {
        let mut flow = AttemptFlow::default();
        flow.enter(Phase::Dying);
        assert_eq!(flow.advance(2.), Next::None);
        assert_eq!(flow.elapsed, 0.);
        for _ in 0..19 {
            assert_eq!(flow.advance(0.05), Next::None);
        }
        assert_eq!(flow.phase, Phase::Dying);
        assert_eq!(flow.advance(0.05), Next::Reset);
        assert_eq!(flow.fade(), 1.);
        assert_eq!(flow.advance(3.), Next::None);
        assert_eq!(flow.phase, Phase::Spawning);
        for _ in 0..5 {
            assert_eq!(flow.advance(0.05), Next::None);
        }
        assert_eq!(flow.advance(0.05), Next::Ready);
        assert!(flow.playing());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::{
        level::{GROUND, Level, LevelObject},
        simulation::{Simulation, Status},
    };
    fn frame(app: &mut App) {
        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_secs_f64(1. / 60.));
        app.world_mut().run_schedule(PreUpdate);
        for _ in 0..4 {
            app.world_mut().run_schedule(FixedUpdate);
        }
    }
    #[test]
    fn fixed_catchup_and_restart_input_cannot_skip_visible_respawn() {
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        let root = std::env::temp_dir().join(format!("gd-attempt-flow-{}", std::process::id()));
        game.root = root.clone();
        game.testing = true;
        game.screen = Screen::Playing;
        let mut level = Level::empty("Respawn regression");
        level.objects.push(LevelObject::new("spike", 40., GROUND));
        game.simulation = Some(Simulation::new(level).unwrap());
        let mut app = App::new();
        app.insert_resource(game)
            .insert_resource(Time::<Real>::default())
            .add_systems(PreUpdate, advance)
            .add_systems(FixedUpdate, super::super::step);
        app.world_mut().run_schedule(FixedUpdate);
        let position = app
            .world()
            .resource::<Game>()
            .simulation
            .as_ref()
            .unwrap()
            .state
            .player
            .position;
        for _ in 0..2400 {
            app.world_mut().run_schedule(FixedUpdate);
        }
        app.world_mut().resource_mut::<Game>().restart();
        assert_eq!(app.world().resource::<Game>().attempt.phase, Phase::Dying);
        assert_eq!(
            app.world()
                .resource::<Game>()
                .simulation
                .as_ref()
                .unwrap()
                .state
                .player
                .position,
            position
        );
        for _ in 0..59 {
            frame(&mut app);
        }
        assert_eq!(app.world().resource::<Game>().attempt.phase, Phase::Dying);
        assert!(matches!(
            app.world()
                .resource::<Game>()
                .simulation
                .as_ref()
                .unwrap()
                .status,
            Status::Dead(_)
        ));
        for _ in 0..3 {
            frame(&mut app);
        }
        assert_eq!(
            app.world().resource::<Game>().attempt.phase,
            Phase::Spawning
        );
        let spawn = app
            .world()
            .resource::<Game>()
            .simulation
            .as_ref()
            .unwrap()
            .state
            .player
            .position;
        for _ in 0..240 {
            app.world_mut().run_schedule(FixedUpdate);
        }
        assert_eq!(
            app.world()
                .resource::<Game>()
                .simulation
                .as_ref()
                .unwrap()
                .state
                .player
                .position,
            spawn
        );
        while !app.world().resource::<Game>().attempt.playing() {
            frame(&mut app);
        }
        assert_eq!(
            app.world()
                .resource::<Game>()
                .simulation
                .as_ref()
                .unwrap()
                .state
                .player
                .position,
            spawn
        );
        // Remove the test hazard so the first live frame can demonstrate normal speed.
        {
            let mut game = app.world_mut().resource_mut::<Game>();
            game.simulation = Some(Simulation::new(Level::empty("Clear")).unwrap());
        }
        frame(&mut app);
        let game = app.world().resource::<Game>();
        let sim = game.simulation.as_ref().unwrap();
        assert_eq!(sim.state.tick, 4);
        assert!(
            (sim.state.player.position.x - spawn.x - crate::constants::SPEED / 60.).abs() < 1e-9
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
