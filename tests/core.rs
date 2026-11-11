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
    s.state.player.position.y = 800.;
    s.state.player.grounded = false;
    run(
        &mut s,
        60,
        TickInput {
            held: true,
            ..Default::default()
        },
    );
    assert!((s.state.player.velocity.y - 500.).abs() < 1e-9);
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
    assert_eq!(s.state.player.position.y, 860.);
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
    profile.save(&root).unwrap();
    assert_eq!(
        Profile::load(&root).unwrap().settings.menu_music_volume,
        100
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn cube_jump_matches_the_current_cpp_profile_parameters() {
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
    // The editable parameters currently match the C++ baseline, restored by the user.
    assert_eq!(ticks, 93);
    assert!((sim.state.player.position.x - origin.x - 348.75).abs() < 1e-6);
    assert!((apex - origin.y - 177.2317708333).abs() < 1e-6);
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
fn ship_free_flight_matches_the_old_cpp_update() {
    let mut sim = Simulation::new(level()).unwrap();
    sim.state.player.mode = PlayerMode::Ship;
    sim.state.player.position.y = 850.;
    sim.state.player.grounded = false;
    sim.state.player.velocity.y = -300.;
    let mut qt_velocity: f64 = 300.;
    let mut qt_y = 0.;
    let mut qt_rotation = 0.;
    for tick in 0..180 {
        let held = tick < 80;
        qt_velocity += 1800. / 240.;
        if held {
            qt_velocity -= 4300. / 240.;
        }
        qt_velocity = qt_velocity.clamp(-500., 500.);
        // ship.cpp advances Y twice, then interpolates toward velocity * 0.07.
        qt_y += 2. * qt_velocity / 240.;
        qt_rotation += (qt_velocity * 0.07 - qt_rotation) * 20. / 240.;
        sim.tick(TickInput {
            held,
            ..Default::default()
        });
        assert!((sim.state.player.velocity.y + qt_velocity).abs() < 1e-8);
        assert!((sim.state.player.position.y - (850. - qt_y)).abs() < 1e-8);
        assert!((sim.state.player.rotation + qt_rotation).abs() < 1e-8);
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
fn ship_takeoff_matches_the_cpp_grounded_gravity_rule() {
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
    assert!((grounded.state.player.velocity.y - 4300. / 240.).abs() < 1e-9);
    assert!((airborne.state.player.velocity.y - 2500. / 240.).abs() < 1e-9);
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
