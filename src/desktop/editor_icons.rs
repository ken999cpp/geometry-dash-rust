//! Small vector UI icons, drawn from nodes so they remain crisp at any UI scale.
use bevy::prelude::*;
#[derive(Clone, Copy)]
pub(super) enum Icon {
    Cursor,
    Pause,
    Build,
    Trash,
    Undo,
    Redo,
    Save,
    SaveAs,
    Folder,
    Copy,
    RotateLeft,
    RotateRight,
    Angle,
    Music,
    Picture,
    Pencil,
    Exit,
    Grid,
    Snap,
    ZoomIn,
    ZoomOut,
    Reset,
    Flag,
    Color,
    Background,
    Ground,
    Hazard,
    Arrow(f32),
    Robot,
    Spider,
    Ball,
}
fn part(commands: &mut Commands, parent: Entity, bundle: impl Bundle) -> Entity {
    let id = commands.spawn(bundle).id();
    commands.entity(parent).add_child(id);
    id
}
fn rect(commands: &mut Commands, parent: Entity, x: f32, y: f32, w: f32, h: f32, filled: bool) {
    part(
        commands,
        parent,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(x),
                top: px(y),
                width: px(w),
                height: px(h),
                border: UiRect::all(px(if filled { 0. } else { 2. })),
                ..default()
            },
            BackgroundColor(if filled { Color::WHITE } else { Color::NONE }),
            BorderColor::all(Color::WHITE),
        ),
    );
}
fn line(commands: &mut Commands, parent: Entity, a: (f32, f32), b: (f32, f32)) {
    let d = Vec2::new(b.0 - a.0, b.1 - a.1);
    let center = Vec2::new((a.0 + b.0) / 2., (a.1 + b.1) / 2.);
    part(
        commands,
        parent,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(center.x - d.length() / 2.),
                top: px(center.y - 1.25),
                width: px(d.length()),
                height: px(2.5),
                border_radius: BorderRadius::all(px(1)),
                ..default()
            },
            BackgroundColor(Color::WHITE),
            UiTransform::from_rotation(Rot2::radians(d.y.atan2(d.x))),
        ),
    );
}
fn path(commands: &mut Commands, parent: Entity, points: &[(f32, f32)]) {
    for pair in points.windows(2) {
        line(commands, parent, pair[0], pair[1]);
    }
}
fn circle(commands: &mut Commands, parent: Entity, x: f32, y: f32, size: f32) {
    part(
        commands,
        parent,
        (
            Node {
                position_type: PositionType::Absolute,
                left: px(x),
                top: px(y),
                width: px(size),
                height: px(size),
                border: UiRect::all(px(2.5)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BorderColor::all(Color::WHITE),
        ),
    );
}
fn arrow(commands: &mut Commands, parent: Entity, angle: f32) {
    let node = part(
        commands,
        parent,
        (
            Node {
                width: px(32),
                height: px(32),
                ..default()
            },
            UiTransform::from_rotation(Rot2::degrees(angle)),
        ),
    );
    path(commands, node, &[(6., 16.), (26., 16.), (18., 8.)]);
    line(commands, node, (26., 16.), (18., 24.));
}
pub(super) fn draw(commands: &mut Commands, parent: Entity, icon: Icon) {
    let node = part(
        commands,
        parent,
        Node {
            width: px(32),
            height: px(32),
            flex_shrink: 0.,
            ..default()
        },
    );
    match icon {
        Icon::Pause => {
            rect(commands, node, 6., 5., 7., 23., true);
            rect(commands, node, 20., 5., 7., 23., true);
        }
        Icon::Cursor => path(
            commands,
            node,
            &[
                (7., 3.),
                (7., 27.),
                (13., 21.),
                (18., 29.),
                (22., 27.),
                (18., 19.),
                (27., 19.),
                (7., 3.),
            ],
        ),
        Icon::Build => {
            rect(commands, node, 5., 5., 22., 22., false);
            line(commands, node, (16., 8.), (16., 24.));
            line(commands, node, (8., 16.), (24., 16.));
        }
        Icon::Trash => {
            path(
                commands,
                node,
                &[(8., 10.), (10., 28.), (23., 28.), (25., 10.)],
            );
            line(commands, node, (5., 8.), (28., 8.));
            path(
                commands,
                node,
                &[(12., 8.), (12., 4.), (21., 4.), (21., 8.)],
            );
            line(commands, node, (14., 13.), (14., 24.));
            line(commands, node, (20., 13.), (20., 24.));
        }
        Icon::Undo | Icon::Redo => {
            if matches!(icon, Icon::Redo) {
                commands
                    .entity(node)
                    .insert(UiTransform::from_scale(Vec2::new(-1., 1.)));
            }
            path(commands, node, &[(11., 7.), (4., 13.), (11., 19.)]);
            path(
                commands,
                node,
                &[(5., 13.), (21., 13.), (27., 18.), (27., 25.), (15., 25.)],
            );
        }
        Icon::Save | Icon::SaveAs => {
            rect(commands, node, 5., 4., 23., 25., false);
            rect(commands, node, 10., 4., 12., 8., false);
            rect(commands, node, 10., 19., 13., 10., false);
            if matches!(icon, Icon::SaveAs) {
                line(commands, node, (26., 15.), (31., 15.));
                line(commands, node, (28., 12.), (28., 18.));
            }
        }
        Icon::Folder => path(
            commands,
            node,
            &[
                (3., 26.),
                (3., 6.),
                (14., 6.),
                (17., 10.),
                (29., 10.),
                (29., 26.),
                (3., 26.),
                (7., 15.),
                (29., 15.),
            ],
        ),
        Icon::Copy => {
            rect(commands, node, 3., 3., 19., 20., false);
            rect(commands, node, 11., 11., 19., 20., false);
        }
        Icon::RotateLeft | Icon::RotateRight => {
            if matches!(icon, Icon::RotateRight) {
                commands
                    .entity(node)
                    .insert(UiTransform::from_scale(Vec2::new(-1., 1.)));
            }
            path(
                commands,
                node,
                &[
                    (7., 7.),
                    (21., 5.),
                    (28., 13.),
                    (26., 24.),
                    (16., 28.),
                    (7., 22.),
                ],
            );
            path(commands, node, &[(7., 2.), (6., 11.), (15., 10.)]);
        }
        Icon::Angle => {
            path(commands, node, &[(6., 4.), (6., 26.), (28., 26.)]);
            line(commands, node, (6., 26.), (26., 7.));
            circle(commands, node, 12., 14., 7.);
        }
        Icon::Music => {
            path(
                commands,
                node,
                &[(11., 25.), (11., 7.), (26., 4.), (26., 22.)],
            );
            line(commands, node, (11., 12.), (26., 9.));
            circle(commands, node, 4., 22., 8.);
            circle(commands, node, 19., 19., 8.);
        }
        Icon::Picture => {
            rect(commands, node, 3., 4., 27., 24., false);
            path(
                commands,
                node,
                &[(5., 25.), (13., 15.), (19., 23.), (24., 17.), (29., 24.)],
            );
            circle(commands, node, 20., 8., 5.);
        }
        Icon::Pencil => {
            path(
                commands,
                node,
                &[
                    (5., 27.),
                    (8., 18.),
                    (23., 3.),
                    (29., 9.),
                    (14., 24.),
                    (5., 27.),
                ],
            );
            line(commands, node, (20., 6.), (26., 12.));
        }
        Icon::Exit => {
            path(
                commands,
                node,
                &[(15., 4.), (5., 4.), (5., 28.), (15., 28.)],
            );
            path(commands, node, &[(11., 16.), (29., 16.), (22., 9.)]);
            line(commands, node, (29., 16.), (22., 23.));
        }
        Icon::Grid => {
            rect(commands, node, 3., 3., 26., 26., false);
            for a in [12., 21.] {
                line(commands, node, (a, 4.), (a, 28.));
                line(commands, node, (4., a), (28., a));
            }
        }
        Icon::Snap => {
            path(
                commands,
                node,
                &[
                    (5., 4.),
                    (5., 21.),
                    (10., 28.),
                    (22., 28.),
                    (27., 21.),
                    (27., 4.),
                    (20., 4.),
                    (20., 20.),
                    (12., 20.),
                    (12., 4.),
                    (5., 4.),
                ],
            );
            line(commands, node, (5., 11.), (12., 11.));
            line(commands, node, (20., 11.), (27., 11.));
        }
        Icon::ZoomIn | Icon::ZoomOut => {
            circle(commands, node, 3., 3., 19.);
            line(commands, node, (20., 20.), (29., 29.));
            line(commands, node, (7., 12.), (18., 12.));
            if matches!(icon, Icon::ZoomIn) {
                line(commands, node, (12., 7.), (12., 18.));
            }
        }
        Icon::Reset => {
            for (a, b, c) in [
                ((3., 12.), (3., 3.), (12., 3.)),
                ((20., 3.), (29., 3.), (29., 12.)),
                ((29., 20.), (29., 29.), (20., 29.)),
                ((12., 29.), (3., 29.), (3., 20.)),
            ] {
                path(commands, node, &[a, b, c]);
            }
            rect(commands, node, 11., 11., 10., 10., false);
        }
        Icon::Flag => {
            path(
                commands,
                node,
                &[(7., 29.), (7., 4.), (27., 4.), (23., 12.), (7., 12.)],
            );
        }
        Icon::Background => {
            rect(commands, node, 3., 3., 26., 26., false);
            circle(commands, node, 8., 7., 7.);
            path(
                commands,
                node,
                &[(4., 26.), (14., 17.), (20., 23.), (27., 15.)],
            );
        }
        Icon::Ground => {
            line(commands, node, (3., 13.), (29., 13.));
            rect(commands, node, 3., 16., 26., 13., false);
            line(commands, node, (12., 16.), (12., 29.));
            line(commands, node, (21., 16.), (21., 29.));
        }
        Icon::Hazard => path(
            commands,
            node,
            &[(3., 28.), (16., 4.), (29., 28.), (3., 28.)],
        ),
        Icon::Color => {
            circle(commands, node, 4., 4., 24.);
            for (x, y) in [(11., 9.), (21., 10.), (10., 20.)] {
                rect(commands, node, x, y, 4., 4., true);
            }
        }
        Icon::Arrow(angle) => arrow(commands, node, angle),
        Icon::Robot => {
            rect(commands, node, 6., 4., 20., 17., false);
            rect(commands, node, 9., 9., 4., 4., true);
            rect(commands, node, 20., 9., 4., 4., true);
            path(commands, node, &[(10., 21.), (10., 28.), (4., 28.)]);
            path(commands, node, &[(23., 21.), (23., 28.), (29., 28.)]);
        }
        Icon::Spider => {
            circle(commands, node, 10., 10., 13.);
            for y in [8., 16., 24.] {
                path(commands, node, &[(3., y), (11., 16.), (23., 16.), (30., y)]);
            }
        }
        Icon::Ball => {
            circle(commands, node, 3., 3., 26.);
            for a in [0_f32, 45., 90., 135.] {
                let a = a.to_radians();
                line(
                    commands,
                    node,
                    (16. - a.cos() * 12., 16. - a.sin() * 12.),
                    (16. + a.cos() * 12., 16. + a.sin() * 12.),
                );
            }
        }
    }
}
