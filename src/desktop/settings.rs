use super::{
    Game,
    ui::{Action, UiRoot, dialog_label},
};
use crate::storage::MusicChannel;
use bevy::{prelude::*, ui::RelativeCursorPosition};

#[derive(Component)]
pub(super) struct MusicSlider(MusicChannel);
#[derive(Component)]
pub(super) struct VolumeVisual {
    channel: MusicChannel,
    value: u8,
    knob: bool,
}
#[derive(Component)]
pub(super) struct VolumeLabel(MusicChannel, u8);

fn node(commands: &mut Commands, parent: Entity, bundle: impl Bundle) -> Entity {
    let child = commands.spawn(bundle).id();
    commands.entity(parent).add_child(child);
    child
}
fn button(
    commands: &mut Commands,
    parent: Entity,
    action: Action,
    text: &str,
    font: &Handle<Font>,
) {
    let button = node(
        commands,
        parent,
        (
            Button,
            action,
            Node {
                min_width: px(48),
                height: px(40),
                padding: UiRect::horizontal(px(12)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.35, 0.18)),
        ),
    );
    dialog_label(commands, button, text, 21., font);
}

pub(super) fn build(commands: &mut Commands, game: &Game, font: &Handle<Font>) {
    let root = commands
        .spawn((
            UiRoot,
            GlobalZIndex(1500),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.75)),
        ))
        .id();
    let panel = node(
        commands,
        root,
        (
            Node {
                width: px(620),
                max_width: percent(94),
                padding: UiRect::all(px(24)),
                flex_direction: FlexDirection::Column,
                row_gap: px(18),
                border: UiRect::all(px(3)),
                border_radius: BorderRadius::all(px(14)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.14, 0.25)),
            BorderColor::all(Color::srgb(0.5, 0.8, 1.)),
        ),
    );
    dialog_label(commands, panel, "НАСТРОЙКИ", 30., font);
    let settings = &game.profile.settings;
    for (label, action, enabled) in [
        (
            "No Death Effect",
            Action::NoDeathEffect,
            settings.no_death_effect,
        ),
        (
            "Hide Checkpoint Buttons",
            Action::HideCheckpointButtons,
            settings.hide_checkpoint_buttons,
        ),
    ] {
        let row = node(
            commands,
            panel,
            Node {
                width: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
        );
        dialog_label(commands, row, label, 20., font);
        button(
            commands,
            row,
            action,
            if enabled { "ON" } else { "OFF" },
            font,
        );
    }
    for (channel, title) in [
        (MusicChannel::Menu, "Музыка в меню"),
        (MusicChannel::Level, "Музыка в уровне"),
    ] {
        dialog_label(commands, panel, title, 20., font);
        let row = node(
            commands,
            panel,
            Node {
                width: percent(100),
                align_items: AlignItems::Center,
                column_gap: px(14),
                ..default()
            },
        );
        button(commands, row, Action::VolumeStep(channel, -5), "−", font);
        let value = settings.volume(channel);
        let slider = node(
            commands,
            row,
            (
                Button,
                MusicSlider(channel),
                RelativeCursorPosition::default(),
                Node {
                    flex_grow: 1.,
                    height: px(24),
                    border_radius: BorderRadius::all(px(12)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.02, 0.04, 0.1)),
            ),
        );
        node(
            commands,
            slider,
            (
                VolumeVisual {
                    channel,
                    value,
                    knob: false,
                },
                Node {
                    position_type: PositionType::Absolute,
                    height: percent(100),
                    width: percent(value as f32),
                    border_radius: BorderRadius::all(px(12)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.85, 0.1)),
            ),
        );
        node(
            commands,
            slider,
            (
                VolumeVisual {
                    channel,
                    value,
                    knob: true,
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: px(16),
                    height: px(34),
                    top: px(-5),
                    left: percent(value as f32),
                    margin: UiRect::left(px(-8)),
                    border_radius: BorderRadius::all(px(5)),
                    ..default()
                },
                BackgroundColor(Color::srgb(1., 0.85, 0.1)),
            ),
        );
        button(commands, row, Action::VolumeStep(channel, 5), "+", font);
        node(
            commands,
            row,
            (
                VolumeLabel(channel, value),
                Text::new(format!("{value}%")),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(20.),
                    ..default()
                },
                Node {
                    width: px(55),
                    ..default()
                },
                TextColor(Color::WHITE),
            ),
        );
    }
    let footer = node(
        commands,
        panel,
        Node {
            justify_content: JustifyContent::Center,
            ..default()
        },
    );
    button(commands, footer, Action::CloseSettings, "Готово", font);
}

pub(super) fn sliders(
    mut game: ResMut<Game>,
    mouse: Res<ButtonInput<MouseButton>>,
    cursors: Query<(&MusicSlider, &Interaction, &RelativeCursorPosition)>,
    mut visuals: Query<(&mut VolumeVisual, &mut Node)>,
    mut labels: Query<(&mut VolumeLabel, &mut Text)>,
    mut dragging: Local<Option<MusicChannel>>,
) {
    if !game.settings_open || game.exit_dialog.is_some() {
        *dragging = None;
        return;
    }
    if mouse.just_pressed(MouseButton::Left) {
        *dragging = cursors
            .iter()
            .find(|(_, interaction, _)| **interaction == Interaction::Pressed)
            .map(|(slider, _, _)| slider.0);
    }
    if mouse.pressed(MouseButton::Left)
        && let Some(channel) = *dragging
        && let Some(position) = cursors
            .iter()
            .find(|(slider, _, _)| slider.0 == channel)
            .and_then(|(_, _, cursor)| cursor.normalized)
    {
        let volume = ((position.x + 0.5) * 100.).round().clamp(0., 100.) as u8;
        game.profile.settings.set_volume(channel, volume);
    }
    if !mouse.pressed(MouseButton::Left) && dragging.take().is_some() {
        game.persist();
    }
    for (mut visual, mut node) in &mut visuals {
        let value = game.profile.settings.volume(visual.channel);
        if visual.value != value {
            if visual.knob {
                node.left = percent(value as f32);
            } else {
                node.width = percent(value as f32);
            }
            visual.value = value;
        }
    }
    for (mut label, mut text) in &mut labels {
        let value = game.profile.settings.volume(label.0);
        if label.1 != value {
            text.0 = format!("{value}%");
            label.1 = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn volume_slider_drags_clamps_and_persists_without_rebuilding_ui() {
        let mut game = Game::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        game.root = std::env::temp_dir().join(format!("gd-slider-{}", std::process::id()));
        let root = game.root.clone();
        game.profile = crate::storage::Profile::default();
        game.open_settings();
        let mut app = App::new();
        app.insert_resource(game)
            .init_resource::<ButtonInput<MouseButton>>()
            .add_systems(Startup, |mut commands: Commands, game: Res<Game>| {
                build(&mut commands, &game, &Handle::default())
            })
            .add_systems(Update, sliders);
        app.update();
        let slider = app
            .world_mut()
            .query::<(Entity, &MusicSlider)>()
            .iter(app.world())
            .find(|(_, s)| s.0 == MusicChannel::Menu)
            .unwrap()
            .0;
        *app.world_mut().get_mut::<Interaction>(slider).unwrap() = Interaction::Pressed;
        app.world_mut()
            .get_mut::<RelativeCursorPosition>(slider)
            .unwrap()
            .normalized = Some(Vec2::new(-0.25, 0.));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert_eq!(
            app.world()
                .resource::<Game>()
                .profile
                .settings
                .menu_music_volume,
            25
        );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        for (x, expected) in [(0.8, 100), (-0.8, 0)] {
            app.world_mut()
                .get_mut::<RelativeCursorPosition>(slider)
                .unwrap()
                .normalized = Some(Vec2::new(x, 0.));
            app.update();
            assert_eq!(
                app.world()
                    .resource::<Game>()
                    .profile
                    .settings
                    .menu_music_volume,
                expected
            );
        }
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.update();
        let saved = crate::storage::Profile::load(&root).unwrap();
        assert_eq!(saved.settings.menu_music_volume, 0);
        assert_eq!(saved.settings.level_music_volume, 100);
        assert!(app.world().get::<MusicSlider>(slider).is_some());
        std::fs::remove_dir_all(root).unwrap();
    }
}
