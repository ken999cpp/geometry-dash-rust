use super::Game;
use crate::storage::FPS_LIMITS;
use bevy::{
    prelude::*,
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
        renderer::{RenderGraph, RenderGraphSystems},
    },
    window::{Monitor, OnMonitor, PrimaryWindow},
};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub(super) struct FrameRatePlugin;

impl Plugin for FrameRatePlugin {
    fn build(&self, app: &mut App) {
        let timing = FrameTiming::default();
        app.insert_resource(timing.clone());
        app.add_systems(Update, sync_display_rate.before(super::FrameSet::World));
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .insert_resource(timing)
                .init_resource::<FramePacer>()
                .insert_resource(RenderFrameLimit(DisplayRate::default().frame_period(60)))
                .add_systems(ExtractSchedule, extract_frame_limit)
                .add_systems(RenderGraph, pace_frames.in_set(RenderGraphSystems::Finish))
                .add_systems(Render, record_present.in_set(RenderSystems::Cleanup));
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct DisplayRate {
    pub refresh_rate_millihertz: Option<u32>,
}

impl DisplayRate {
    pub fn supports(self, fps: u32) -> bool {
        // Display modes such as 59.94 / 143.98 Hz use the nominal 60 / 144 label.
        // The actual frame interval below still respects the exact refresh rate.
        let nominal_hz = self
            .refresh_rate_millihertz
            .unwrap_or(60_000)
            .saturating_add(500)
            / 1000;
        FPS_LIMITS.contains(&fps) && fps <= nominal_hz
    }

    pub fn selected_fps(self, preferred: u32) -> u32 {
        FPS_LIMITS
            .into_iter()
            .rev()
            .find(|&fps| fps <= preferred && self.supports(fps))
            .unwrap_or(60)
    }

    fn frame_period(self, preferred: u32) -> Duration {
        let millihertz = (self.selected_fps(preferred) * 1000)
            .min(self.refresh_rate_millihertz.unwrap_or(60_000).max(1));
        // Round upward so the limiter cannot exceed even a fractional refresh rate.
        Duration::from_nanos(1_000_000_000_000_u64.div_ceil(u64::from(millihertz)))
    }
}

fn sync_display_rate(
    mut game: ResMut<Game>,
    windows: Query<&OnMonitor, With<PrimaryWindow>>,
    monitors: Query<&Monitor>,
) {
    let refresh_rate_millihertz = windows
        .single()
        .ok()
        .and_then(|on_monitor| monitors.get(on_monitor.0).ok())
        .and_then(|monitor| monitor.refresh_rate_millihertz)
        .filter(|&rate| rate > 0);
    let display = DisplayRate {
        refresh_rate_millihertz,
    };
    if game.display_rate != display {
        game.display_rate = display;
        game.ui_dirty = true;
    }
    // Wait for monitor detection before changing a saved preference at startup.
    let fps = display.selected_fps(game.profile.settings.fps_limit());
    if refresh_rate_millihertz.is_some() && game.profile.settings.fps_limit != fps {
        game.profile.settings.fps_limit = fps;
        game.persist();
        game.ui_dirty = true;
    }
}

#[derive(Resource)]
struct RenderFrameLimit(Duration);

#[derive(Resource, Default)]
struct FramePacer {
    last_gate: Option<Instant>,
}

/// Timings come from the renderer after present, not the pipelined main loop.
#[derive(Resource, Clone, Default)]
pub(super) struct FrameTiming(Arc<Mutex<VecDeque<Instant>>>);
impl FrameTiming {
    pub fn fps(&self) -> f64 {
        let samples = self.0.lock().unwrap();
        match (samples.front(), samples.back()) {
            (Some(first), Some(last))
                if samples.len() > 1 && last.elapsed() < Duration::from_secs(1) =>
            {
                (samples.len() - 1) as f64 / last.duration_since(*first).as_secs_f64()
            }
            _ => 0.,
        }
    }
    fn record(&self, now: Instant) {
        let mut samples = self.0.lock().unwrap();
        samples.push_back(now);
        while samples.len() > 2 && now.duration_since(samples[0]) > Duration::from_secs(1) {
            samples.pop_front();
        }
    }
}

fn wait_until(deadline: Instant) {
    // Windows sleep may overshoot a short 4.17 ms frame budget. Sleep the bulk
    // and use a bounded fine wait for the last millisecond.
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining > Duration::from_millis(1) {
        std::thread::sleep(remaining - Duration::from_millis(1));
    }
    while Instant::now() < deadline {
        std::hint::spin_loop();
    }
}

fn extract_frame_limit(mut commands: Commands, game: Extract<Res<Game>>) {
    commands.insert_resource(RenderFrameLimit(
        game.display_rate
            .frame_period(game.profile.settings.fps_limit()),
    ));
}

fn pace_frames(limit: Res<RenderFrameLimit>, mut pacer: ResMut<FramePacer>) {
    // GPU work is already submitted. Wait immediately before presenting, so main
    // thread work and pipelined rendering cannot produce a burst above the limit.
    if let Some(previous) = pacer.last_gate {
        wait_until(previous + limit.0);
    }
    // Anchor at the presentation gate. Cleanup and swap-chain work belong in
    // the next frame budget; adding them after every wait lowered actual FPS.
    pacer.last_gate = Some(Instant::now());
}

fn record_present(timing: Res<FrameTiming>) {
    // Measure from the completed present, including any VSync wait. Slow frames
    // start a fresh interval instead of allowing catch-up frames above the cap.
    timing.record(Instant::now());
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{ecs::schedule::ScheduleLabel, render::extract_plugin::ExtractPlugin};

    #[test]
    fn fps_uses_presented_intervals_and_does_not_report_single_frame_jitter() {
        let timing = FrameTiming::default();
        let origin = Instant::now() - Duration::from_secs(2);
        for i in 0..=480 {
            timing.record(
                origin
                    + Duration::from_nanos(i * 4_166_667)
                    + Duration::from_micros(if i % 2 == 0 { 0 } else { 200 }),
            );
        }
        assert!((timing.fps() - 240.).abs() < 0.1);
        let slower = FrameTiming::default();
        for i in 0..=180 {
            slower.record(
                Instant::now() - Duration::from_secs(1) + Duration::from_secs_f64(i as f64 / 180.),
            );
        }
        assert!((slower.fps() - 180.).abs() < 1.);
    }

    #[test]
    #[ignore = "wall-clock performance measurement for the current computer"]
    fn measure_240_fps_presentation_gate() {
        let period = Duration::from_nanos(4_166_667);
        let mut samples = Vec::new();
        let mut gate = Instant::now();
        for _ in 0..480 {
            std::thread::sleep(Duration::from_micros(800));
            wait_until(gate + period);
            let now = Instant::now();
            samples.push(now.duration_since(gate).as_secs_f64());
            gate = now;
        }
        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        samples.sort_by(f64::total_cmp);
        println!(
            "240 FPS gate: average={:.3} FPS, median={:.4} ms, p99={:.4} ms, max={:.4} ms",
            1. / mean,
            samples[240] * 1000.,
            samples[475] * 1000.,
            samples[479] * 1000.
        );
        assert!(samples[0] >= period.as_secs_f64());
    }

    #[derive(Resource, Default)]
    struct PresentTimes(Vec<Instant>);

    #[derive(Resource, Default)]
    struct SimulatedRenderWork(Duration);

    fn simulate_render_and_present(world: &mut World) {
        std::thread::sleep(world.resource::<SimulatedRenderWork>().0);
        world.run_schedule(RenderGraph);
        world.resource_mut::<PresentTimes>().0.push(Instant::now());
    }

    #[test]
    fn rendered_frames_obey_live_settings_without_bursts_after_a_slow_frame() {
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.profile = crate::storage::Profile::default();
        let mut app = App::new();
        app.insert_resource(game)
            .add_plugins(ExtractPlugin::default());
        let display = app.world_mut().spawn(monitor(Some(240_000))).id();
        app.world_mut()
            .spawn((Window::default(), PrimaryWindow, OnMonitor(display)));
        let render_app = app.sub_app_mut(RenderApp);
        render_app.update_schedule = Some(Render.intern());
        render_app
            .add_schedule(RenderGraph::base_schedule())
            .init_resource::<PresentTimes>()
            .init_resource::<SimulatedRenderWork>()
            .add_systems(
                Render,
                simulate_render_and_present.in_set(RenderSystems::Render),
            );
        app.add_plugins(FrameRatePlugin);

        for fps in [240, 60, 144, 120] {
            app.world_mut()
                .resource_mut::<Game>()
                .profile
                .settings
                .fps_limit = fps;
            let expected_period = DisplayRate {
                refresh_rate_millihertz: Some(240_000),
            }
            .frame_period(fps);
            for frame in 0..8 {
                app.sub_app_mut(RenderApp)
                    .world_mut()
                    .resource_mut::<SimulatedRenderWork>()
                    .0 = Duration::from_millis(if frame == 2 { 30 } else { 0 });
                app.update();
                let render_world = app.sub_app(RenderApp).world();
                assert_eq!(
                    render_world.resource::<RenderFrameLimit>().0,
                    expected_period
                );
                let times = &render_world.resource::<PresentTimes>().0;
                if times.len() >= 2 {
                    let interval = times[times.len() - 1].duration_since(times[times.len() - 2]);
                    assert!(
                        interval + Duration::from_micros(100) >= expected_period,
                        "{fps} FPS: frame presented after {interval:?}"
                    );
                }
            }
        }
    }

    fn monitor(rate: Option<u32>) -> Monitor {
        Monitor {
            name: None,
            physical_height: 1080,
            physical_width: 1920,
            physical_position: IVec2::ZERO,
            refresh_rate_millihertz: rate,
            scale_factor: 1.,
            video_modes: vec![],
        }
    }

    #[test]
    fn frame_limits_respect_current_refresh_rate_including_fractional_and_slow_displays() {
        for (rate, supported) in [
            (None, vec![60]),
            (Some(48_000), vec![]),
            (Some(59_940), vec![60]),
            (Some(60_000), vec![60]),
            (Some(75_000), vec![60]),
            (Some(120_000), vec![60, 120]),
            (Some(143_000), vec![60, 120]),
            (Some(143_980), vec![60, 120, 144]),
            (Some(165_000), vec![60, 120, 144]),
            (Some(239_760), vec![60, 120, 144, 240]),
            (Some(360_000), vec![60, 120, 144, 240]),
        ] {
            let display = DisplayRate {
                refresh_rate_millihertz: rate,
            };
            assert_eq!(
                FPS_LIMITS
                    .into_iter()
                    .filter(|&fps| display.supports(fps))
                    .collect::<Vec<_>>(),
                supported,
            );
            assert!(!display.supports(90));
            for preference in FPS_LIMITS {
                let actual_fps = 1. / display.frame_period(preference).as_secs_f64();
                assert!(actual_fps <= f64::from(preference));
                assert!(actual_fps <= f64::from(rate.unwrap_or(60_000)) / 1000.);
            }
        }
    }

    #[test]
    fn moving_the_window_to_a_slower_monitor_clamps_and_saves_the_limit() {
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.root = std::env::temp_dir().join(format!("gd-fps-monitor-{}", std::process::id()));
        let root = game.root.clone();
        game.profile = crate::storage::Profile::default();
        game.profile.settings.fps_limit = 240;
        let mut app = App::new();
        app.insert_resource(game)
            .add_systems(Update, sync_display_rate);
        let fast = app.world_mut().spawn(monitor(Some(240_000))).id();
        let slow = app.world_mut().spawn(monitor(Some(60_000))).id();
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        app.update();
        // Startup without monitor information must not discard a saved choice.
        assert_eq!(
            app.world().resource::<Game>().profile.settings.fps_limit,
            240
        );
        app.world_mut().entity_mut(window).insert(OnMonitor(fast));
        app.update();
        assert_eq!(
            app.world().resource::<Game>().profile.settings.fps_limit,
            240
        );
        app.world_mut().entity_mut(window).insert(OnMonitor(slow));
        app.update();
        assert_eq!(
            app.world().resource::<Game>().profile.settings.fps_limit,
            60
        );
        assert_eq!(
            crate::storage::Profile::load(&root)
                .unwrap()
                .settings
                .fps_limit,
            60
        );
        assert_eq!(
            app.world()
                .resource::<Game>()
                .display_rate
                .refresh_rate_millihertz,
            Some(60_000)
        );
        // An invalid report uses the conservative fallback instead of dividing by zero.
        app.world_mut()
            .get_mut::<Monitor>(slow)
            .unwrap()
            .refresh_rate_millihertz = Some(0);
        app.update();
        assert_eq!(
            app.world().resource::<Game>().display_rate,
            DisplayRate::default()
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
