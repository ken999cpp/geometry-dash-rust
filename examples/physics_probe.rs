//! Run without a window: cargo run --no-default-features --example physics_probe
use geometry_dash::{
    constants,
    level::{GROUND, Level, LevelObject},
    simulation::{PlayerMode, Simulation, TickInput},
};

fn empty_track() -> Level {
    let mut level = Level::empty("Physics probe");
    level
        .objects
        .push(LevelObject::new("block", 1_000_000., GROUND));
    level
}

fn main() {
    if std::env::args().any(|a| a == "--stereo") {
        let level = Level::load(std::path::Path::new("levels/Stereo Madness.json")).unwrap();
        let mut sim = Simulation::new(level).unwrap();
        let x = std::env::args()
            .find_map(|a| a.strip_prefix("--x=").and_then(|v| v.parse::<f64>().ok()))
            .unwrap_or(11950.);
        let y = std::env::args()
            .find_map(|a| a.strip_prefix("--y=").and_then(|v| v.parse::<f64>().ok()))
            .unwrap_or(1000.);
        sim.state.player.position = geometry_dash::geometry::Point::new(x, y);
        sim.state.player.previous = sim.state.player.position;
        sim.state.player.grounded = std::env::args().any(|a| a == "--grounded");
        sim.state.player.velocity.y = if sim.state.player.grounded { 0. } else { -800. };
        for _ in 0..1200 {
            let was = sim.state.player.grounded;
            sim.tick(TickInput {
                held: true,
                ..Default::default()
            });
            let p = &sim.state.player;
            if p.grounded != was || sim.status != geometry_dash::simulation::Status::Running {
                println!(
                    "x={:.2} y={:.2} vy={:.2} grounded={} status={:?}",
                    p.position.x, p.position.y, p.velocity.y, p.grounded, sim.status
                );
            }
            if sim.status != geometry_dash::simulation::Status::Running || p.position.x > 13080. {
                break;
            }
        }
        return;
    }
    println!(
        "Hz={} speed={} gravity={} jump={} cube_rotation={} deg/s",
        constants::HZ,
        constants::SPEED,
        constants::GRAVITY,
        constants::JUMP,
        constants::CUBE_ROTATION_SPEED
    );
    let mut cube = Simulation::new(empty_track()).expect("valid probe track");
    let origin = cube.state.player.position;
    cube.tick(TickInput {
        pressed: true,
        ..Default::default()
    });
    let mut apex = cube.state.player.position.y;
    while !cube.state.player.grounded && cube.state.elapsed < 10. {
        cube.tick(Default::default());
        apex = apex.max(cube.state.player.position.y);
    }
    if !cube.state.player.grounded {
        eprintln!("Cube did not land within 10 seconds; check the chosen parameters.");
        return;
    }
    println!(
        "Cube: ticks={} time={:.6}s range={:.6} height={:.6} landing_rotation={:.3} deg",
        cube.state.tick,
        cube.state.elapsed,
        cube.state.player.position.x - origin.x,
        apex - origin.y,
        cube.state.player.rotation
    );
    for _ in 0..10 {
        cube.tick(Default::default());
    }
    println!(
        "Cube rotation after 10 grounded ticks: {:.3} deg",
        cube.state.player.rotation
    );

    let mut ship = Simulation::new(empty_track()).expect("valid probe track");
    ship.state.player.mode = PlayerMode::Ship;
    ship.state.player.position.y = (GROUND + constants::CEILING) / 2.;
    ship.state.player.previous = ship.state.player.position;
    ship.state.player.grounded = false;
    let mut elapsed = 0.;
    for held in [true, true, false, false] {
        let ticks = (constants::HZ * 0.1).round() as usize;
        for _ in 0..ticks {
            ship.tick(TickInput {
                held,
                ..Default::default()
            });
        }
        elapsed += ticks as f64 * constants::DT;
        let p = &ship.state.player;
        println!(
            "Ship: t={elapsed:.3}s held={held} y={:.3} vy={:.3} angle={:.3} deg",
            p.position.y, p.velocity.y, p.rotation
        );
    }
}
