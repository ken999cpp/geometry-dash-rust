use geometry_dash::{
    editor::Editor,
    geometry::{Point, Rect},
    level::{GROUND, Level, LevelObject},
    simulation::{CEILING, DT, PlayerMode, Simulation, Status, TickInput},
    storage::Profile,
};
use std::path::Path;

fn level() -> Level {
    let mut l = Level::empty("Test");
    l.objects.push(LevelObject::new("block", 100000., GROUND));
    l
}

#[test]
fn stereo_madness_ship_start_uses_the_portal_corridor_and_keeps_blocks_fixed() {
    let mut track =
        Level::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("levels/Stereo Madness.json"))
            .unwrap();
    let blocks = track.objects.clone();
    track
        .objects
        .push(LevelObject::new("start_pos", 23000., 820.));
    let marker = track.objects.len() - 1;
    let mut sim = Simulation::new(track).unwrap();
    sim.start_at(marker).unwrap();
    assert_eq!(sim.state.player.mode, PlayerMode::Ship);
    assert_eq!(sim.state.floor, 420.);
    assert_eq!(sim.state.ceiling, 1220.);
    assert!((geometry_dash::constants::GAME_CAMERA_Y + sim.state.camera_y - 820.).abs() < 1e-8);
    assert!((sim.state.elapsed - (23000. / geometry_dash::constants::SPEED)).abs() < 1e-8);
    let start = sim.state.clone();
    run(
        &mut sim,
        15,
        TickInput {
            held: true,
            ..Default::default()
        },
    );
    assert_eq!(&sim.level.objects[..marker], blocks.as_slice());
    sim.restart(false);
    assert_eq!(sim.state, start);
    let normal = Simulation::new(sim.level.clone()).unwrap();
    assert_eq!(normal.state.player.mode, PlayerMode::Cube);
    assert_eq!(normal.progress(), 0);
}

#[test]
fn same_mode_portal_changes_corridor_without_resetting_momentum() {
    let mut track = level();
    track
        .objects
        .push(LevelObject::new("ship_portal", 0., 1140.));
    let mut sim = Simulation::new(track).unwrap();
    let mut reference = Simulation::new(level()).unwrap();
    sim.state.player.mode = PlayerMode::Ship;
    sim.state.player.position.y = 1260.;
    sim.state.player.previous = sim.state.player.position;
    sim.state.player.grounded = false;
    sim.state.ceiling = 1800.;
    reference.state = sim.state.clone();
    sim.tick(Default::default());
    reference.tick(Default::default());
    assert_eq!(sim.state.corridor_floor, 820.);
    assert_eq!(sim.state.ceiling, 1620.);
    assert_eq!(sim.state.player.velocity, reference.state.player.velocity);
}

#[test]
fn start_position_restores_trigger_time_and_explicit_mode_gravity() {
    let mut track = level();
    let mut trigger = LevelObject::new("trigger", 100., 0.);
    trigger.trigger_type = Some("background_color_change".into());
    trigger
        .trigger_params
        .insert("color".into(), "#ff0000".into());
    trigger.trigger_params.insert("duration".into(), 2.into());
    track.objects.push(trigger);
    let mut marker = LevelObject::new("start_pos", 950., 700.);
    marker.extra.insert("mode".into(), "wave".into());
    marker.extra.insert("gravity".into(), "reverse".into());
    track.objects.push(marker);
    let mut sim = Simulation::new(track).unwrap();
    sim.start_at(2).unwrap();
    assert_eq!(sim.state.player.mode, PlayerMode::Wave);
    assert_eq!(sim.state.player.gravity, 1.);
    assert!(sim.state.active[1]);
    assert!(sim.state.background.current[0] > 0.12 && sim.state.background.current[0] < 1.);
    assert_eq!(sim.state.player.previous, sim.state.player.position);
    let saved = sim.state.clone();
    sim.tick(Default::default());
    sim.restart(false);
    assert_eq!(sim.state, saved);
}

#[test]
fn start_markers_are_editable_and_do_not_extend_level_length() {
    let mut editor = Editor::new(level(), None);
    let length = editor.level.length();
    editor.choose_tool(18);
    editor.place(Point::new(200000., 800.));
    assert_eq!(editor.level.length(), length);
    assert_eq!(editor.level.objects.last().unwrap().kind, "start_pos");
    editor.cycle_start_setting(false);
    editor.cycle_start_setting(true);
    let object = editor.level.objects.last().unwrap();
    assert_eq!(object.extra["mode"], "cube");
    assert_eq!(object.extra["gravity"], "normal");
    editor.undo();
    assert_eq!(
        editor.level.objects.last().unwrap().extra["gravity"],
        "auto"
    );
    editor.redo();
    let restored: Level =
        serde_json::from_slice(&serde_json::to_vec(&editor.level).unwrap()).unwrap();
    assert_eq!(restored, editor.level);
}
fn run(sim: &mut Simulation, ticks: usize, input: TickInput) {
    for _ in 0..ticks {
        sim.tick(input);
    }
}

#[test]
fn original_levels_roundtrip_without_data_loss() {
    for name in ["Stereo Madness", "Back On Track", "Polargeist"] {
        let l =
            Level::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("levels/{name}.json")))
                .unwrap();
        l.validate_playable().unwrap();
        let encoded = serde_json::to_vec(&l).unwrap();
        let decoded: Level = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(l, decoded);
        assert!(l.objects.len() > 1000);
    }
}
#[test]
fn unknown_fields_survive_editing_but_gameplay_rejects_unknown_objects() {
    let value = serde_json::json!({"name":"Future","version":3,"objects":[{"type":"future","x":10,"y":20,"custom":{"a":1}}]});
    let l: Level = serde_json::from_value(value).unwrap();
    assert!(l.validate_playable().is_err());
    let encoded = serde_json::to_value(&l).unwrap();
    assert_eq!(encoded["version"], 3);
    assert_eq!(encoded["objects"][0]["custom"]["a"], 1);
}
#[test]
fn old_asset_paths_resolve() {
    for p in [
        "assets/cube/Cube001.png",
        "qrc:/assets/cube/Cube001.png",
        ":/assets/cube/Cube001.png",
    ] {
        assert_eq!(geometry_dash::level::asset_path(p), "cube/Cube001.png");
    }
    assert_eq!(
        geometry_dash::level::asset_path("user://custom.png"),
        "user://custom.png"
    );
}
#[test]
fn collision_respects_rotated_rectangles_and_edges() {
    let a = Rect::new(Point::new(0., 0.), Point::new(80., 80.), 0.);
    assert!(!a.overlaps(Rect::new(Point::new(80., 0.), Point::new(80., 80.), 0.)));
    assert!(a.overlaps(Rect::new(Point::new(79., 0.), Point::new(80., 80.), 0.)));
    let thin = Rect::new(Point::new(0., 0.), Point::new(200., 10.), 45.);
    assert!(!thin.overlaps(Rect::new(Point::new(65., -65.), Point::new(10., 10.), 0.)));
    assert!(thin.contains(Point::new(30., 30.)));
}
#[test]
fn vertical_span_uses_rotated_surface() {
    let r = Rect::new(Point::new(0., 0.), Point::new(80., 80.), 45.);
    let (lo, hi) = geometry_dash::simulation::vertical_span(r, 0.).unwrap();
    assert!((hi - 80. / 2_f64.sqrt()).abs() < 1e-6);
    assert!((lo + hi).abs() < 1e-6);
}
#[test]
fn cube_jumps_and_lands() {
    let mut s = Simulation::new(level()).unwrap();
    s.tick(TickInput {
        held: true,
        pressed: true,
        ..Default::default()
    });
    assert!(s.state.player.position.y > GROUND + 40.);
    run(&mut s, 150, TickInput::default());
    assert!(s.state.player.grounded);
    assert_eq!(s.state.player.position.y, GROUND + 40.);
}
#[test]
fn cube_hold_repeats_jump_on_landing() {
    let mut s = Simulation::new(level()).unwrap();
    run(
        &mut s,
        110,
        TickInput {
            held: true,
            ..Default::default()
        },
    );
    assert!(s.state.player.position.y > GROUND + 40.);
}
#[test]
fn ship_thrust_and_speed_caps() {
    let mut s = Simulation::new(level()).unwrap();
    s.state.player.mode = PlayerMode::Ship;
    s.state.player.position.y = 500.;
    s.state.player.grounded = false;
    run(
        &mut s,
        120,
        TickInput {
            held: true,
            ..Default::default()
        },
    );
    assert!(
        (s.state.player.velocity.y - geometry_dash::constants::SHIP_MAX_RISE_SPEED).abs() < 1e-9
    );
    assert!(s.state.player.velocity.y <= geometry_dash::constants::SHIP_MAX_RISE_SPEED);
    run(&mut s, 150, TickInput::default());
    assert!(s.state.player.velocity.y >= -geometry_dash::constants::SHIP_MAX_FALL_SPEED);
}
#[test]
fn wave_has_constant_diagonal_speed() {
    let mut s = Simulation::new(level()).unwrap();
    s.state.player.mode = PlayerMode::Wave;
    s.state.player.position.y = 800.;
    s.tick(TickInput {
        held: true,
        ..Default::default()
    });
    let v = s.state.player.velocity.y;
    s.tick(TickInput {
        held: true,
        ..Default::default()
    });
    assert_eq!(v, s.state.player.velocity.y);
    s.tick(TickInput::default());
    assert_eq!(v, -s.state.player.velocity.y);
}

#[test]
fn stereo_madness_18_percent_hold_lands_on_the_leading_edge_and_continues() {
    let track =
        Level::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("levels/Stereo Madness.json"))
            .unwrap();
    let mut sim = Simulation::new(track).unwrap();
    // Press while falling toward the lower platform, then keep holding through
    // the landing and both upper platforms. The old center-only collision
    // missed the first upper landing before x=12240.
    sim.state.player.position = Point::new(11950., 1000.);
    sim.state.player.previous = sim.state.player.position;
    sim.state.player.velocity.y = -800.;
    sim.state.player.grounded = false;
    let mut landings = Vec::new();
    for _ in 0..500 {
        let was = sim.state.player.grounded;
        sim.tick(TickInput {
            held: true,
            ..Default::default()
        });
        assert_eq!(
            sim.status,
            Status::Running,
            "failed at {:?}",
            sim.state.player.position
        );
        if !was && sim.state.player.grounded && sim.state.player.position.y > 1100. {
            landings.push(sim.state.player.position);
            if landings.len() == 2 {
                break;
            }
        }
    }
    assert_eq!(landings.len(), 2);
    assert!(
        landings[0].x < 12240.,
        "the leading edge must support the landing"
    );
    assert!((12840. ..13080.).contains(&landings[1].x));
}

#[test]
fn pads_and_yellow_orbs_use_native_impulses_in_every_mode_and_gravity() {
    use geometry_dash::constants::GD_VELOCITY_UNIT;
    for mode in PlayerMode::ALL {
        for gravity in [-1., 1.] {
            for kind in ["jump_pad", "yellow_orb"] {
                let mut track = level();
                track.objects.push(LevelObject::new(
                    kind,
                    if kind == "jump_pad" { 5. } else { 0. },
                    if kind == "jump_pad" { 700. } else { 660. },
                ));
                let mut sim = Simulation::new(track).unwrap();
                let p = &mut sim.state.player;
                p.mode = mode;
                p.gravity = gravity;
                p.position = Point::new(40., 700.);
                p.previous = p.position;
                p.grounded = false;
                sim.tick(TickInput {
                    held: true,
                    pressed: true,
                    ..Default::default()
                });
                if mode == PlayerMode::Wave && kind == "jump_pad" {
                    assert!(!sim.state.active[1]);
                    continue;
                }
                let native = if kind == "jump_pad" {
                    if matches!(mode, PlayerMode::Ball | PlayerMode::Spider) {
                        9.6
                    } else {
                        16.
                    }
                } else {
                    match mode {
                        PlayerMode::Ball | PlayerMode::Spider => 7.826,
                        PlayerMode::Robot => 10.062,
                        _ => 11.18,
                    }
                };
                assert!(sim.state.active[1], "{mode:?} {kind}");
                assert!(
                    (sim.state.player.velocity.y + gravity * native * GD_VELOCITY_UNIT).abs()
                        < 1e-8,
                    "{mode:?} {kind}"
                );
                assert!(sim.state.player.robot_boost_invalidated);
            }
        }
    }
}

#[test]
fn every_mode_portal_preserves_the_native_fraction_of_momentum() {
    for old in PlayerMode::ALL {
        for new in PlayerMode::ALL {
            let mut track = level();
            track.objects.push(LevelObject::new(
                &format!("{}_portal", new.name()),
                -10.,
                580.,
            ));
            let mut sim = Simulation::new(track).unwrap();
            let mut baseline = Simulation::new(level()).unwrap();
            let p = &mut sim.state.player;
            p.mode = old;
            p.position.y = 700.;
            p.previous = p.position;
            p.velocity.y = 300.;
            p.grounded = false;
            p.rotation = 31.;
            baseline.state.player = p.clone();
            baseline.tick(Default::default());
            sim.tick(Default::default());
            let mut expected = baseline.state.player.velocity.y;
            if old != new {
                if matches!(old, PlayerMode::Ship | PlayerMode::Wave) {
                    expected *= 0.5;
                }
                if matches!(new, PlayerMode::Ship | PlayerMode::Wave) {
                    expected *= 0.5;
                }
            }
            assert_eq!(sim.state.player.mode, new);
            assert!(
                (sim.state.player.velocity.y - expected).abs() < 1e-8,
                "{old:?} -> {new:?}"
            );
        }
    }
}

#[test]
fn gravity_portal_halves_momentum_only_when_gravity_changes() {
    for (kind, expected_factor, gravity) in [
        ("ReverseGravityPortal", 0.5, 1.),
        ("NormalGravityPortal", 1., -1.),
    ] {
        let mut track = level();
        track.objects.push(LevelObject::new(kind, 0., 580.));
        let mut sim = Simulation::new(track).unwrap();
        sim.state.player.position.y = 700.;
        sim.state.player.previous = sim.state.player.position;
        sim.state.player.velocity.y = 900.;
        sim.state.player.grounded = false;
        let mut baseline = Simulation::new(level()).unwrap();
        baseline.state.player = sim.state.player.clone();
        baseline.tick(Default::default());
        sim.tick(Default::default());
        assert_eq!(sim.state.player.gravity, gravity);
        assert_eq!(
            sim.state.player.velocity.y,
            baseline.state.player.velocity.y * expected_factor
        );
    }
}

#[test]
fn ground_action_modes_buffer_a_fresh_airborne_press_until_landing() {
    for mode in [PlayerMode::Robot, PlayerMode::Ball, PlayerMode::Spider] {
        let mut sim = Simulation::new(level()).unwrap();
        let p = &mut sim.state.player;
        p.mode = mode;
        p.position.y = 490.;
        p.previous = p.position;
        p.grounded = false;
        p.velocity.y = -100.;
        sim.tick(TickInput {
            held: true,
            pressed: true,
            ..Default::default()
        });
        for _ in 0..100 {
            let had_landed = sim.state.player.grounded;
            sim.tick(TickInput {
                held: true,
                ..Default::default()
            });
            if had_landed {
                break;
            }
        }
        assert!(!sim.state.player.orb_armed, "{mode:?}");
        if mode == PlayerMode::Robot {
            assert!(sim.state.player.velocity.y > 0.);
        } else {
            assert_eq!(sim.state.player.gravity, 1.);
        }
    }
}

#[test]
fn robot_release_permanently_ends_that_jumps_boost() {
    let mut sim = Simulation::new(level()).unwrap();
    sim.state.player.mode = PlayerMode::Robot;
    sim.tick(TickInput {
        held: true,
        pressed: true,
        ..Default::default()
    });
    run(
        &mut sim,
        5,
        TickInput {
            held: true,
            ..Default::default()
        },
    );
    sim.tick(Default::default());
    let before = sim.state.player.velocity.y;
    sim.tick(TickInput {
        held: true,
        ..Default::default()
    });
    assert!(sim.state.player.robot_boost_invalidated);
    assert!(sim.state.player.velocity.y < before);
}

#[test]
fn wave_trace_retains_subframe_spam_corners_and_boundary_intersections() {
    let mut sim = Simulation::new(level()).unwrap();
    sim.state.player.mode = PlayerMode::Wave;
    sim.state.player.position.y = 700.;
    sim.state.player.previous = sim.state.player.position;
    for i in 0..120 {
        sim.tick(TickInput {
            held: i % 2 == 0,
            ..Default::default()
        });
    }
    assert_eq!(sim.wave_trail.len(), 121);
    for (a, b) in sim.wave_trail.iter().zip(sim.wave_trail.iter().skip(1)) {
        assert!(((b.x - a.x) - (b.y - a.y).abs()).abs() < 1e-7);
    }
    run(&mut sim, 240, TickInput::default());
    for (a, b) in sim.wave_trail.iter().zip(sim.wave_trail.iter().skip(1)) {
        let d = *b - *a;
        assert!(d.y.abs() < 1e-7 || (d.x - d.y.abs()).abs() < 1e-7);
    }
    assert!(sim.state.player.rotation.abs() < 0.03);
    sim.practice = true;
    sim.checkpoint();
    sim.restart(true);
    assert!(sim.wave_trail.is_empty());
}
#[test]
fn robot_hold_increases_jump_height() {
    let mut short = Simulation::new(level()).unwrap();
    short.state.player.mode = PlayerMode::Robot;
    let mut long = Simulation::new(level()).unwrap();
    long.state.player.mode = PlayerMode::Robot;
    let input = TickInput {
        held: true,
        pressed: true,
        ..Default::default()
    };
    short.tick(input);
    long.tick(input);
    run(&mut short, 40, TickInput::default());
    run(
        &mut long,
        40,
        TickInput {
            held: true,
            ..Default::default()
        },
    );
    assert!(long.state.player.position.y > short.state.player.position.y + 20.);
}
#[test]
fn ball_switches_gravity_once_per_press() {
    let mut s = Simulation::new(level()).unwrap();
    s.state.player.mode = PlayerMode::Ball;
    s.tick(TickInput {
        pressed: true,
        held: true,
        ..Default::default()
    });
    assert_eq!(s.state.player.gravity, 1.);
    let y = s.state.player.position.y;
    run(
        &mut s,
        10,
        TickInput {
            held: true,
            ..Default::default()
        },
    );
    assert_eq!(s.state.player.gravity, 1.);
    assert!(s.state.player.position.y > y);
}
#[test]
fn spider_teleports_to_nearest_surface() {
    let mut l = level();
    l.objects.push(LevelObject::new("block", 0., 900.));
    let mut s = Simulation::new(l).unwrap();
    s.state.player.mode = PlayerMode::Spider;
    s.tick(TickInput {
        pressed: true,
        ..Default::default()
    });
    assert_eq!(s.state.player.gravity, 1.);
    assert_eq!(s.state.player.position.y, 864.);
    assert!(s.state.player.grounded);
}
#[test]
fn spider_cannot_teleport_through_hazards() {
    let mut l = level();
    l.objects.push(LevelObject::new("special_object", 0., 700.));
    let mut s = Simulation::new(l).unwrap();
    s.state.player.mode = PlayerMode::Spider;
    s.tick(TickInput {
        pressed: true,
        ..Default::default()
    });
    assert!(matches!(s.status, Status::Dead(_)));
}
#[test]
fn portal_only_activates_once() {
    let mut l = level();
    l.objects.push(LevelObject::new("ship_portal", 0., GROUND));
    let mut s = Simulation::new(l).unwrap();
    s.tick(TickInput::default());
    assert_eq!(s.state.player.mode, PlayerMode::Ship);
    assert!(s.state.active[1]);
    s.state.player.mode = PlayerMode::Cube;
    s.tick(TickInput::default());
    assert_eq!(s.state.player.mode, PlayerMode::Cube);
}
#[test]
fn pad_and_orb_apply_single_impulse() {
    for kind in ["yellow_orb", "jump_pad"] {
        let mut l = level();
        l.objects.push(LevelObject::new(kind, 0., GROUND));
        let mut s = Simulation::new(l).unwrap();
        s.tick(TickInput {
            held: true,
            ..Default::default()
        });
        assert!(s.state.active[1]);
        assert!(s.state.player.velocity.y > 1000.);
    }
}

#[test]
fn holding_a_ground_jump_does_not_activate_an_orb_crossed_in_flight() {
    let mut l = level();
    l.objects
        .push(LevelObject::new("yellow_orb", 190., GROUND + 100.));
    let orb = l.objects[1].rect();
    let mut sim = Simulation::new(l).unwrap();
    let mut touched = false;
    for _ in 0..100 {
        sim.tick(TickInput {
            held: true,
            ..Default::default()
        });
        touched |= sim.state.player.danger().overlaps(orb);
        assert!(
            !sim.events
                .contains(&geometry_dash::simulation::GameEvent::Orb)
        );
    }
    assert!(touched, "The test jump must actually pass through the orb");
    assert!(!sim.state.active[1]);
}

#[test]
fn a_fresh_airborne_hold_buffers_one_orb_and_survives_checkpoint_restore() {
    let mut l = level();
    l.objects
        .push(LevelObject::new("yellow_orb", 190., GROUND + 100.));
    let mut sim = Simulation::new(l).unwrap();
    sim.practice = true;
    let held = TickInput {
        held: true,
        ..Default::default()
    };
    run(&mut sim, 20, held);
    sim.tick(TickInput::default());
    sim.tick(TickInput {
        pressed: true,
        ..held
    });
    assert!(sim.state.player.orb_armed);
    assert!(!sim.state.active[1]);
    sim.checkpoint();
    let mut activations = 0;
    for _ in 0..35 {
        sim.tick(held);
        activations += usize::from(
            sim.events
                .contains(&geometry_dash::simulation::GameEvent::Orb),
        );
    }
    assert_eq!(activations, 1);
    assert!(sim.state.active[1]);
    assert!(!sim.state.player.orb_armed);
    let expected = sim.state.clone();
    sim.restart(true);
    assert!(sim.state.player.orb_armed);
    run(&mut sim, 35, held);
    assert_eq!(sim.state, expected);
}

#[test]
fn one_held_press_cannot_chain_two_orbs_without_a_new_press() {
    let mut l = level();
    l.objects.push(LevelObject::new("yellow_orb", 0., GROUND));
    l.objects.push(LevelObject::new("yellow_orb", 0., GROUND));
    let mut sim = Simulation::new(l).unwrap();
    let held = TickInput {
        held: true,
        ..Default::default()
    };
    sim.tick(held);
    assert!(sim.state.active[1]);
    assert!(!sim.state.active[2]);
    sim.tick(held);
    assert!(!sim.state.active[2]);
    sim.tick(TickInput::default());
    sim.tick(TickInput {
        pressed: true,
        ..held
    });
    assert!(sim.state.active[2]);
}
#[test]
fn death_freezes_the_attempt_and_waits_a_full_second() {
    let mut l = level();
    l.objects.push(LevelObject::new("spike", 100., GROUND));
    let mut s = Simulation::new(l).unwrap();
    while s.status == Status::Running {
        s.tick(TickInput::default());
    }
    assert!(matches!(s.status, Status::Dead(_)));
    let at_death = s.state.clone();
    run(&mut s, 120, TickInput::default());
    assert!(matches!(s.status, Status::Dead(_)));
    assert_eq!(s.state, at_death);
    run(&mut s, 119, TickInput::default());
    assert!(matches!(s.status, Status::Dead(_)));
    run(&mut s, 2, TickInput::default());
    assert_eq!(s.status, Status::Running);
    assert!(s.state.player.position.x < 100.);
}
#[test]
fn checkpoint_restores_all_simulation_state() {
    let mut s = Simulation::new(level()).unwrap();
    s.practice = true;
    s.state.player.mode = PlayerMode::Ship;
    s.state.player.position.y = 800.;
    run(
        &mut s,
        30,
        TickInput {
            held: true,
            ..Default::default()
        },
    );
    s.checkpoint();
    let expected = s.state.clone();
    run(&mut s, 50, TickInput::default());
    s.restart(true);
    assert_eq!(s.state.player.position, expected.player.position);
    assert_eq!(s.state.player.mode, expected.player.mode);
    assert_eq!(s.state.active, expected.active);
    assert_eq!(s.state.background, expected.background);
    assert_eq!(s.state.elapsed, expected.elapsed);
    assert_eq!(s.state.camera_y, expected.camera_y);
    assert_eq!(s.state.floor, expected.floor);
}
#[test]
fn checkpoint_removal_and_normal_restart() {
    let mut s = Simulation::new(level()).unwrap();
    s.practice = true;
    s.checkpoint();
    s.remove_checkpoint();
    assert!(s.checkpoints.is_empty());
    run(&mut s, 100, TickInput::default());
    s.restart(true);
    assert_eq!(s.state.elapsed, 0.);
}
#[test]
fn color_transition_checkpoint_preserves_timeline() {
    let mut l = level();
    let mut o = LevelObject::new("trigger", 0., 0.);
    o.trigger_type = Some("background_color_change".into());
    o.trigger_params.insert("color".into(), "#ff0000".into());
    o.trigger_params.insert("duration".into(), 2.into());
    l.objects.push(o);
    let mut s = Simulation::new(l).unwrap();
    s.practice = true;
    run(&mut s, 50, TickInput::default());
    s.checkpoint();
    run(&mut s, 100, TickInput::default());
    let expected = s.state.background;
    s.restart(true);
    run(&mut s, 100, TickInput::default());
    assert_eq!(s.state.background, expected);
}
fn replay(fps: u32) -> geometry_dash::simulation::Snapshot {
    let mut s = Simulation::new(level()).unwrap();
    let mut accumulator = 0.;
    for _ in 0..fps * 4 {
        accumulator += 1. / fps as f64;
        while accumulator + 1e-10 >= DT {
            let tick = s.state.tick;
            s.tick(TickInput {
                held: tick % 180 < 25,
                pressed: tick.is_multiple_of(180),
                released: tick % 180 == 25,
            });
            accumulator -= DT;
        }
    }
    s.state
}
#[test]
fn rendering_fps_cannot_change_simulation() {
    let expected = replay(30);
    assert_eq!(expected.tick, 960);
    for fps in [60, 144, 240] {
        assert_eq!(expected, replay(fps), "{fps} FPS");
    }
}
#[test]
fn ball_reaches_ceiling() {
    let mut s = Simulation::new(level()).unwrap();
    s.state.player.mode = PlayerMode::Ball;
    s.tick(TickInput {
        pressed: true,
        ..Default::default()
    });
    run(&mut s, 130, TickInput::default());
    assert_eq!(s.state.player.position.y, CEILING - 40.);
    assert!(s.state.player.grounded);
}
#[test]
fn level_completion_uses_last_object_plus_indent() {
    let mut s = Simulation::new(Level::empty("Empty")).unwrap();
    run(&mut s, 150, TickInput::default());
    assert_eq!(s.status, Status::Complete);
}
#[test]
fn editor_undo_roundtrip_and_history_limit() {
    let mut e = Editor::new(Level::empty("Editor"), None);
    e.tool = 1;
    e.place(Point::new(100., 450.));
    assert_eq!(e.level.objects[0].x, 80.);
    e.duplicate();
    e.rotate(90.);
    e.move_selected(Point::new(80., 0.), true);
    e.delete();
    e.undo();
    assert_eq!(e.level.objects.len(), 2);
    e.undo();
    e.undo();
    e.undo();
    assert_eq!(e.level.objects.len(), 1);
    e.undo();
    assert!(e.level.objects.is_empty());
    for i in 0..60 {
        e.place(Point::new(i as f64 * 80., GROUND));
    }
    for _ in 0..60 {
        e.undo();
    }
    assert_eq!(e.level.objects.len(), 10);
}
#[test]
fn save_failure_preserves_editor_dirty_state() {
    let mut e = Editor::new(Level::empty("Editor"), None);
    e.tool = 1;
    e.place(Point::new(0., GROUND));
    let parent = std::env::temp_dir().join(format!("gd-save-failure-{}", std::process::id()));
    std::fs::write(&parent, b"not a directory").unwrap();
    assert!(e.save(parent.join("level.json")).is_err());
    assert!(e.dirty);
    std::fs::remove_file(parent).unwrap();
}
#[test]
fn imported_progress_does_not_override_existing_profile() {
    let mut p = Profile::default();
    let mut l = level();
    l.normal_progress = 80;
    p.import(&l);
    l.normal_progress = 10;
    p.import(&l);
    assert_eq!(p.levels["Test"].normal, 80);
}

#[test]
fn a_tap_between_render_frames_still_jumps() {
    let mut sim = Simulation::new(level()).unwrap();
    sim.tick(TickInput {
        held: false,
        pressed: true,
        released: true,
    });
    assert!(sim.state.player.velocity.y > 0.);
}

#[test]
fn selecting_a_rotated_object_preserves_its_angle_when_rotating() {
    let mut l = Level::empty("Rotated");
    let mut o = LevelObject::new("block", 0., GROUND);
    o.rotation = 90.;
    l.objects.push(o);
    let mut e = Editor::new(l, None);
    e.select_at(Point::new(40., GROUND + 40.));
    e.rotate(2.5);
    assert_eq!(e.level.objects[0].rotation, 92.5);
    e.undo();
    assert_eq!(e.level.objects[0].rotation, 90.);
}

#[test]
fn profile_and_level_save_roundtrip() {
    let root = std::env::temp_dir().join(format!("gd-persistence-test-{}", std::process::id()));
    let mut p = Profile::default();
    p.import(&level());
    p.cube_icon = 1;
    p.save(&root).unwrap();
    let restored = Profile::load(&root).unwrap();
    assert_eq!(restored.cube_icon, 1);
    assert!(restored.levels.contains_key("Test"));
    let path = root.join("levels/custom.json");
    level().save(&path).unwrap();
    assert_eq!(Level::load(&path).unwrap(), level());
    level().save(&path).unwrap(); // Replacing a file must work on both supported platforms.
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn old_profiles_gain_default_settings_and_completed_level_stars() {
    use geometry_dash::storage::{MusicChannel, Settings};
    let root = std::env::temp_dir().join(format!("gd-settings-compat-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("profile.json"),
        r#"{"cube_icon":1,"settings":{"show_star_counter":false},"levels":{"Stereo Madness":{"normal":100,"practice":100,"attempts":8}}}"#,
    )
    .unwrap();
    let mut profile = Profile::load(&root).unwrap();
    assert_eq!(profile.settings, Settings::default());
    profile.import(&Level::empty("Stereo Madness"));
    profile.import(&Level::empty("Stereo Madness"));
    assert_eq!(profile.total_stars(), 1);
    assert_eq!(profile.levels["Stereo Madness"].attempts, 8);
    let mut practice = Level::empty("Back On Track");
    practice.practice_progress = 100;
    profile.import(&practice);
    assert_eq!(profile.total_stars(), 1);
    profile.settings.no_death_effect = true;
    profile.settings.hide_checkpoint_buttons = true;
    profile.settings.set_volume(MusicChannel::Menu, 37);
    profile.settings.set_volume(MusicChannel::Level, 0);
    profile.settings.fps_limit = 144;
    profile.save(&root).unwrap();
    let restored = Profile::load(&root).unwrap();
    assert_eq!(restored.settings, profile.settings);
    assert!(
        serde_json::to_value(&restored).unwrap()["settings"]
            .get("show_star_counter")
            .is_none()
    );
    assert_eq!(restored.total_stars(), 1);
    profile.settings.menu_music_volume = 250;
    profile.settings.fps_limit = 999;
    profile.save(&root).unwrap();
    assert_eq!(
        Profile::load(&root).unwrap().settings.menu_music_volume,
        100
    );
    assert_eq!(Profile::load(&root).unwrap().settings.fps_limit, 240);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn cube_jump_matches_the_original_stereo_madness_capture() {
    let mut sim = Simulation::new(level()).unwrap();
    let origin = sim.state.player.position;
    sim.tick(TickInput {
        pressed: true,
        ..Default::default()
    });
    let mut apex = sim.state.player.position.y;
    let mut ticks = 1;
    while !sim.state.player.grounded && ticks < 200 {
        sim.tick(TickInput::default());
        ticks += 1;
        apex = apex.max(sim.state.player.position.y);
    }
    assert_eq!(ticks, 103);
    assert!((sim.state.player.position.x - origin.x - 356.5861236).abs() < 1e-6);
    assert!((apex - origin.y - 170.2584).abs() < 1e-6);

    // Measured from the user's Steam game on 2026-11-12. Screen height excluding
    // the window border was 1042 px (320 native units). The first spike supplies
    // a world-distance clock, independent of screenshot or input latency.
    let px_per_unit = 1042. / geometry_dash::constants::VIEWPORT_HEIGHT;
    let captured = [
        (1444., 644.),
        (1355.5, 548.5),
        (1271., 523.),
        (1195., 553.),
        (1106., 654.),
    ];
    let mut replay = Simulation::new(level()).unwrap();
    let mut previous = replay.state.player.position;
    for (spike_x, screen_y) in captured {
        let target_x = origin.x + (1494.85947157 - spike_x) / px_per_unit;
        while replay.state.player.position.x < target_x {
            previous = replay.state.player.position;
            replay.tick(TickInput {
                pressed: replay.state.tick == 0,
                ..Default::default()
            });
        }
        let current = replay.state.player.position;
        let alpha = (target_x - previous.x) / (current.x - previous.x);
        let y = previous.y + (current.y - previous.y) * alpha;
        let projected_y = 730. - (y - origin.y) * px_per_unit;
        assert!(
            (projected_y - screen_y).abs() < 1.5,
            "captured {screen_y}, replayed {projected_y}"
        );
    }
}

#[test]
fn editor_trigger_markers_are_circular_and_preserve_activation_coordinates() {
    let mut level = level();
    for channel in [
        "background_color_change",
        "ground_color_change",
        "color_change",
    ] {
        let mut trigger = LevelObject::new("trigger", 800., 0.);
        trigger.trigger_type = Some(channel.into());
        level.objects.push(trigger);
    }
    let original = level.clone();
    let mut editor = Editor::new(level, None);
    for i in 1..=3 {
        let marker = editor.level.objects[i].editor_rect();
        assert_eq!(
            marker.center,
            Point::new(820., 1000. + (i - 1) as f64 * 88.)
        );
        editor.select_at(marker.center);
        assert_eq!(editor.selected, Some(i));
        assert!(!editor.level.objects[i].editor_contains(marker.center + Point::new(40., 40.)));
        assert!(!editor.level.objects[i].editor_contains(Point::new(820., 20.)));
        editor.move_selected(Point::new(0., 80.), true);
        assert_eq!(editor.level.objects[i], original.objects[i]);
    }
    // Marker placement is presentation only, including after a file round trip.
    let encoded = serde_json::to_vec(&editor.level).unwrap();
    assert_eq!(serde_json::from_slice::<Level>(&encoded).unwrap(), original);
    editor.move_selected(Point::new(80., 240.), true);
    assert_eq!(editor.level.objects[3].x, 880.);
    assert_eq!(editor.level.objects[3].y, 0.);
    editor.undo();
    assert_eq!(editor.level, original);
}

#[test]
fn ship_uses_native_quantized_acceleration_and_a_single_displacement() {
    let mut sim = Simulation::new(level()).unwrap();
    sim.state.player.mode = PlayerMode::Ship;
    sim.state.player.position.y = 650.;
    sim.state.player.grounded = false;
    // Native reference: 0.225 delta, gravity 0.958199, three-decimal Y velocity;
    // held factors 0.5 / 0.4, released factors 0.32 / 0.48 near the apex.
    let mut native_velocity: f64 = 0.;
    let mut native_y = 0.;
    for tick in 0..144 {
        let held = tick < 48;
        let falling = native_velocity < 1.916398;
        let acceleration = match (held, falling) {
            (true, true) => 0.5,
            (true, false) => 0.4,
            (false, true) => -0.32,
            (false, false) => -0.48,
        };
        native_velocity =
            ((native_velocity + 0.958199 * 0.225 * acceleration) * 1000.).round() / 1000.;
        native_velocity = native_velocity.clamp(-6.4, 8.);
        native_y += native_velocity * 0.225;
        sim.tick(TickInput {
            held,
            ..Default::default()
        });
        assert!((sim.state.player.velocity.y - native_velocity * 144.).abs() < 1e-8);
        assert!((sim.state.player.position.y - (650. + native_y * (80. / 30.))).abs() < 1e-8);
    }
}

#[test]
fn ship_response_is_symmetric_under_reversed_gravity() {
    let mut normal = Simulation::new(level()).unwrap();
    normal.state.player.mode = PlayerMode::Ship;
    normal.state.player.position.y = 800.;
    normal.state.player.grounded = false;
    let mut reversed = Simulation::new(level()).unwrap();
    reversed.state.player.mode = PlayerMode::Ship;
    reversed.state.player.position.y = 800.;
    reversed.state.player.grounded = false;
    reversed.state.player.gravity = 1.;
    for tick in 0..80 {
        let input = TickInput {
            held: tick < 40,
            ..Default::default()
        };
        normal.tick(input);
        reversed.tick(input);
        assert!((normal.state.player.velocity.y + reversed.state.player.velocity.y).abs() < 1e-8);
    }
}

#[test]
fn ship_takeoff_uses_the_same_thrust_as_airborne_reversal() {
    let mut grounded = Simulation::new(level()).unwrap();
    grounded.state.player.mode = PlayerMode::Ship;
    let mut airborne = Simulation::new(level()).unwrap();
    airborne.state.player.mode = PlayerMode::Ship;
    airborne.state.player.position.y = 800.;
    airborne.state.player.grounded = false;
    let input = TickInput {
        held: true,
        ..Default::default()
    };
    grounded.tick(input);
    airborne.tick(input);
    assert!(!grounded.state.player.grounded);
    assert!(grounded.state.player.velocity.y > 0.);
    assert!((grounded.state.player.velocity.y - 15.552).abs() < 1e-9);
    assert_eq!(
        grounded.state.player.velocity.y,
        airborne.state.player.velocity.y
    );
}

#[test]
fn held_auto_jump_keeps_the_original_extra_takeoff_step() {
    let mut sim = Simulation::new(level()).unwrap();
    let held = TickInput {
        held: true,
        ..Default::default()
    };
    sim.tick(held);
    let fresh = sim.state.player.velocity.y;
    for _ in 0..103 {
        sim.tick(held);
    }
    assert!(sim.state.player.velocity.y > fresh);
    assert!((sim.state.player.velocity.y - geometry_dash::constants::JUMP).abs() < 1e-6);
}

#[test]
fn wave_displacement_is_exactly_45_degrees_and_falling_has_a_terminal_speed() {
    let mut wave = Simulation::new(level()).unwrap();
    wave.state.player.mode = PlayerMode::Wave;
    wave.state.player.position.y = 700.;
    let origin = wave.state.player.position;
    wave.tick(TickInput {
        held: true,
        ..Default::default()
    });
    let delta = wave.state.player.position - origin;
    assert!((delta.x - delta.y).abs() < 1e-9);
    let mut cube = Simulation::new(level()).unwrap();
    cube.state.player.position.y = 100_000.;
    cube.state.player.grounded = false;
    run(&mut cube, 480, TickInput::default());
    assert_eq!(
        cube.state.player.velocity.y,
        -geometry_dash::constants::MAX_FALL_SPEED
    );
}

#[test]
fn ship_terminal_speeds_are_reached_in_both_directions() {
    let mut up = Simulation::new(level()).unwrap();
    up.state.player.mode = PlayerMode::Ship;
    up.state.player.position.y = 600.;
    up.state.player.grounded = false;
    run(
        &mut up,
        100,
        TickInput {
            held: true,
            ..Default::default()
        },
    );
    assert!(
        (up.state.player.velocity.y - geometry_dash::constants::SHIP_MAX_RISE_SPEED).abs() < 1e-8
    );
    let mut down = Simulation::new(level()).unwrap();
    down.state.player.mode = PlayerMode::Ship;
    down.state.player.position.y = 1200.;
    down.state.player.grounded = false;
    run(&mut down, 100, TickInput::default());
    assert!(
        (down.state.player.velocity.y + geometry_dash::constants::SHIP_MAX_FALL_SPEED).abs() < 1e-8
    );
}

#[test]
fn editor_redo_and_new_edit_branch_are_consistent() {
    let mut editor = Editor::new(Level::empty("Undo redo"), None);
    editor.place(Point::new(80., GROUND));
    editor.duplicate();
    let duplicated = editor.level.clone();
    editor.undo();
    assert_eq!(editor.level.objects.len(), 1);
    editor.redo();
    assert_eq!(editor.level, duplicated);
    editor.undo();
    editor.place(Point::new(320., GROUND));
    let branched = editor.level.clone();
    editor.redo();
    assert_eq!(editor.level, branched);
}

#[test]
fn editor_mode_switching_keeps_the_build_tool_and_snap_is_independent() {
    use geometry_dash::editor::EditorMode;
    let mut editor = Editor::new(Level::empty("Modes"), None);
    editor.choose_tool(10);
    editor.set_mode(EditorMode::Edit);
    assert_eq!(editor.tool, 0);
    editor.set_mode(EditorMode::Delete);
    editor.set_mode(EditorMode::Build);
    assert_eq!(editor.tool, 10);
    editor.grid = false;
    assert_eq!(editor.snap(Point::new(95., 450.)), Point::new(80., 400.));
    editor.snap_to_grid = false;
    assert_eq!(editor.snap(Point::new(95., 450.)), Point::new(95., 450.));
}

#[test]
fn editor_box_selection_clipboard_and_delete_preserve_data_and_undo_atomically() {
    let mut doc = Level::empty("Clipboard");
    let mut a = LevelObject::new("block", 80., 400.);
    a.extra
        .insert("custom_data".into(), serde_json::json!({"keep":[1,2,3]}));
    doc.objects = vec![
        a,
        LevelObject::new("spike", 240., 480.),
        LevelObject::new("block", 1600., 400.),
    ];
    let original = doc.clone();
    let mut editor = Editor::new(doc, None);
    editor.select_rect(Point::new(0., 350.), Point::new(400., 600.));
    assert_eq!(editor.selection(), vec![0, 1]);
    editor.copy();
    editor.camera = Point::new(800., 800.);
    editor.paste();
    assert_eq!(editor.selection(), vec![3, 4]);
    assert_eq!(editor.level.objects[4].x - editor.level.objects[3].x, 160.);
    assert_eq!(editor.level.objects[4].y - editor.level.objects[3].y, 80.);
    assert_eq!(editor.level.objects[3].extra, original.objects[0].extra);
    let pasted = editor.level.clone();
    editor.delete();
    assert_eq!(editor.level, original);
    editor.undo();
    assert_eq!(editor.level, pasted);
    editor.undo();
    assert_eq!(editor.level, original);
    editor.redo();
    assert_eq!(editor.level, pasted);
}

#[test]
fn editor_scale_layers_and_flips_match_collision_geometry() {
    let mut editor = Editor::new(Level::empty("Transforms"), None);
    editor.choose_tool(4);
    editor.place(Point::new(80., 400.));
    let before = editor.level.objects[0].rect().center;
    let old_hitbox = editor.level.objects[0].hitbox();
    editor.set_scale(2., 0.5);
    let object = &editor.level.objects[0];
    assert_eq!(object.rect().center, before);
    assert_eq!(
        object.hitbox().half,
        Point::new(old_hitbox.half.x * 2., old_hitbox.half.y * 0.5)
    );
    let offset = object.hitbox().center.y - before.y;
    editor.flip(false);
    assert!((editor.level.objects[0].hitbox().center.y - before.y + offset).abs() < 1e-10);
    editor.layer = 3;
    editor.show_all_layers = false;
    assert!(!editor.object_visible(&editor.level.objects[0]));
    editor.place(Point::new(400., 400.));
    assert!(editor.object_visible(&editor.level.objects[1]));
    editor.select_rect(Point::new(0., 0.), Point::new(2000., 2000.));
    assert_eq!(editor.selection(), vec![1]);
}

#[test]
fn initial_level_colors_survive_restart_and_invalid_colors_use_defaults() {
    let mut doc = level();
    doc.extra
        .insert("background_color".into(), "#4080ff".into());
    doc.extra.insert("ground_color".into(), "#invalid".into());
    let mut sim = Simulation::new(doc).unwrap();
    let expected = [64. / 255., 128. / 255., 1., 1.];
    assert_eq!(sim.state.background.current, expected);
    assert_eq!(sim.state.ground.current, [0.03, 0.08, 0.4, 1.]);
    run(&mut sim, 10, TickInput::default());
    sim.restart(false);
    assert_eq!(sim.state.background.current, expected);
}

#[test]
fn editor_bulk_delete_respects_layer_and_filter_and_undo_restores_every_object() {
    let mut doc = Level::empty("Delete filters");
    let mut hidden = LevelObject::new("block", 0., 400.);
    hidden.extra.insert("editor_layer".into(), 3.into());
    doc.objects = vec![
        LevelObject::new("block", 0., 400.),
        LevelObject::new("spike", 80., 400.),
        LevelObject::new("start_pos", 160., 400.),
        hidden,
    ];
    let original = doc.clone();
    let mut editor = Editor::new(doc, None);
    editor.delete_filter = Some("solid".into());
    editor.delete_all(false);
    assert_eq!(editor.level.objects.len(), 3);
    assert_eq!(editor.level.objects[0].kind, "spike");
    editor.undo();
    assert_eq!(editor.level, original);
    editor.delete_all(true);
    assert_eq!(editor.level.objects.len(), 3);
    assert!(!editor.level.objects.iter().any(|o| o.kind == "start_pos"));
    editor.undo();
    assert_eq!(editor.level, original);
}
