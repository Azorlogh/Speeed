use bevy::{
	log::LogPlugin,
	prelude::*,
	window::{WindowMode, WindowResolution},
};
use bevy_ecs_tilemap::prelude::TilemapRenderSettings;
use bevy_egui::{
	egui::{self, FontDefinitions},
	EguiContexts,
};
use bevy_hanabi::HanabiPlugin;
#[allow(unused)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

mod game;
mod input;
mod leaderboard;
mod level;
mod menu;
mod player;
mod replay;
mod settings;
mod states;

// Pour débogguer l'ordre d'exécution des systèmes
const DEBUG_SCHEDULE: bool = false;

fn main() {
	let mut app = App::new();

	if DEBUG_SCHEDULE {
		app.add_plugins(
			DefaultPlugins
				.build()
				.disable::<LogPlugin>()
				.set(WindowPlugin {
					primary_window: Some(Window {
						resizable: false,
						mode: WindowMode::Windowed,
						resolution: WindowResolution::new(1280., 720.)
							.with_scale_factor_override(0.8),

						..default()
					}),
					..default()
				}),
		);
	} else {
		app.add_plugins(
			DefaultPlugins
				.build()
				.add_before::<bevy::asset::AssetPlugin, _>(
					bevy_embedded_assets::EmbeddedAssetPlugin,
				),
		);
	}

	app
		// Background color
		.insert_resource(ClearColor(Color::BLACK))
		// Fixed timesteps
		.insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
		// This is just to fix a bug within bevy_ecs_tilemap `https://github.com/StarArawn/bevy_ecs_tilemap/issues/373`
		.insert_resource(TilemapRenderSettings {
			render_chunk_size: UVec2::new(128, 128),
		})
		// Particle effects
		.add_plugin(HanabiPlugin)
		// Physics
		.insert_resource(RapierConfiguration {
			gravity: -Vec2::Y * 80.0,
			timestep_mode: TimestepMode::Variable {
				max_dt: 1.0 / 60.0,
				time_scale: 1.0,
				substeps: 4,
			},
			..default()
		})
		.add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0))
		// User interface
		.add_plugin(bevy_egui::EguiPlugin)
		// settings
		.add_plugin(settings::SettingsPlugin)
		// input
		.add_plugin(input::InputPlugin)
		// Game state (menu, in-game, etc)
		.add_plugin(states::StatePlugin)
		// Player logic
		.add_plugin(player::PlayerPlugin)
		// Levels and objects inside them
		.add_plugin(level::LevelPlugin)
		// Replays & ghosts
		.add_plugin(replay::ReplayPlugin)
		// Main menu logic
		.add_plugin(menu::MenuPlugin)
		// Main game logic
		.add_plugin(game::GamePlugin)
		// Leaderboard view (menu after a successful run)
		.add_plugin(leaderboard::LeaderboardPlugin)
		.add_startup_systems((configure_egui, setup_music));

	#[cfg(debug_assertions)]
	{
		// app.add_plugin(WorldInspectorPlugin::new());
		// app.add_plugin(RapierDebugRenderPlugin::default());
	}

	if DEBUG_SCHEDULE {
		// bevy_mod_debugdump::print_main_schedule(&mut app);
	}

	app.run();
}

/// Allows controlling the music
#[derive(Resource)]
pub struct MusicSink(Handle<AudioSink>);
fn setup_music(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	audio_sinks: Res<Assets<AudioSink>>,
	audio: Res<Audio>,
) {
	let music = asset_server.load("sounds/music.ogg");
	let mut music_sink = audio.play_with_settings(
		music,
		PlaybackSettings {
			repeat: true,
			volume: 0.0,
			speed: 1.0,
		},
	);
	music_sink.make_strong(&audio_sinks);
	commands.insert_resource(MusicSink(music_sink));
}

/// UI styling
fn configure_egui(mut contexts: EguiContexts) {
	let ctx = contexts.ctx_mut();
	let mut fonts = FontDefinitions::default();
	// normal text
	{
		let mut f =
			egui::FontData::from_static(include_bytes!("../assets/fonts/FiraSans-Bold.ttf"));
		f.tweak.scale = 2.0;
		fonts.font_data.insert("normal".to_owned(), f);
		fonts
			.families
			.entry(egui::FontFamily::Proportional)
			.or_default()
			.insert(0, "normal".to_owned());
	}

	ctx.set_fonts(fonts);
}
