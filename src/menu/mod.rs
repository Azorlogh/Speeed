mod settings;

use bevy::{
	app::prelude::*,
	ecs::prelude::*,
	prelude::{Assets, AudioSink, AudioSinkPlayback, Camera2dBundle, Handle},
	utils::default,
};
use bevy_ecs_ldtk::{prelude::LdtkFields, LdtkAsset, LevelSelection, LevelSet, Respawn};
use bevy_iced::{
	iced::{
		widget::{text, Button, Column, Row, Scrollable, TextInput},
		Alignment,
	},
	IcedContext, IcedPlugin,
};

use self::settings::{SettingsMenuState, SettingsUiMessage};
use crate::{
	leaderboard::{Leaderboard, Nickname},
	states::{AppState, Exit},
	MusicSink,
};

#[derive(Clone)]
pub enum UiMessage {
	EnterLevel(usize),
	EnterSettings,
	SetNickname(String),
	SetMusicMuted(bool),
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugin(IcedPlugin)
			.add_event::<UiMessage>()
			.add_event::<SettingsUiMessage>()
			.insert_resource(MenuState::default())
			.add_system(setup.in_schedule(OnEnter(AppState::Menu)))
			.add_systems((ui_system, ui_update).distributive_run_if(in_state(AppState::Menu)))
			.add_systems(
				(settings::settings_update, settings::settings_ui)
					.distributive_run_if(in_state(AppState::Menu)),
			);
	}
}

fn ui_update(
	mut messages: EventReader<UiMessage>,
	mut commands: Commands,
	mut state: ResMut<MenuState>,
	mut next_app_state: ResMut<NextState<AppState>>,
	mut level_selection: ResMut<LevelSelection>,
	mut nickname: ResMut<Nickname>,
	q_ldtk_world: Query<Entity, With<LevelSet>>,
	audio_sinks: Res<Assets<AudioSink>>,
	music: Res<MusicSink>,
) {
	let world = q_ldtk_world.single();

	for msg in messages.iter() {
		match msg {
			UiMessage::EnterLevel(idx) => {
				*level_selection = LevelSelection::Index(*idx);
				next_app_state.set(AppState::Game);
				commands.entity(world).insert(Respawn);
			}
			UiMessage::EnterSettings => {
				*state = MenuState::Settings(SettingsMenuState::default());
			}
			UiMessage::SetNickname(s) => nickname.0 = s.clone(),
			UiMessage::SetMusicMuted(muted) => {
				if let Some(sink) = audio_sinks.get(&music.0) {
					if *muted {
						sink.pause();
					} else {
						sink.play();
					}
				}
			}
		}
	}
}

fn ui_system(
	mut ctx: IcedContext<UiMessage>,
	state: Res<MenuState>,
	leaderboard: Res<Leaderboard>,
	q_ldtk_world: Query<&Handle<LdtkAsset>, With<LevelSet>>,
	ldtk_asset: Res<Assets<LdtkAsset>>,
	nickname: Res<Nickname>,
	audio_sinks: Res<Assets<AudioSink>>,
	music: Res<MusicSink>,
) {
	let MenuState::Main = *state else {
		return;
	};

	let ldtk_handle = q_ldtk_world.single();

	let title = text(format!("S P E E E D")).size(38.0);
	let mut levels = Row::new().spacing(16.0);
	for (i, level) in ldtk_asset
		.get(ldtk_handle)
		.unwrap()
		.project
		.levels
		.iter()
		.enumerate()
	{
		let name = level.get_string_field("name").unwrap();
		levels = levels.push(
			Button::new(
				Column::new()
					.push(text(format!("Level {i}")))
					.push(text(name))
					.push("Best time:")
					.push(
						if let Some((name, score)) = leaderboard.get_scores(i).first() {
							text(format!("{name}: {score}").as_str())
						} else {
							text("no score yet")
						},
					),
			)
			.on_press(UiMessage::EnterLevel(i))
			.padding(16.0),
		);
	}

	let mut extra_buttons = Row::new();

	// Music mute/unmute
	if let Some(sink) = audio_sinks.get(&music.0) {
		match sink.is_paused() {
			false => {
				extra_buttons = extra_buttons
					.push(Button::new("Mute music").on_press(UiMessage::SetMusicMuted(true)))
			}
			true => {
				extra_buttons = extra_buttons
					.push(Button::new("Unmute music").on_press(UiMessage::SetMusicMuted(false)))
			}
		}
	}

	let main_col = Column::new()
		.align_items(Alignment::Center)
		.padding(64.0)
		.spacing(32.0)
		.push(title)
		.push(TextInput::new("Nickname", &nickname.0).on_input(|s| UiMessage::SetNickname(s)))
		.push(Scrollable::new(levels.padding([32.0, 0.0])).horizontal_scroll(default()))
		.push(Button::new("Settings").on_press(UiMessage::EnterSettings))
		.push(extra_buttons);

	ctx.display(main_col);
}

#[derive(Default, Resource)]
pub enum MenuState {
	#[default]
	Main,
	Settings(SettingsMenuState),
}

fn setup(mut commands: Commands) {
	let mut camera = Camera2dBundle::default();
	camera.transform.translation.z = -10000.0;
	commands.spawn((camera, Exit(AppState::Menu)));
}
