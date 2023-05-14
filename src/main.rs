use bevy::{
	log::LogPlugin,
	prelude::*,
	window::{WindowMode, WindowResolution},
};
use bevy_hanabi::HanabiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

mod editor;
mod game;
mod input;
mod leaderboard;
mod level;
mod menu;
mod player;
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
		app.add_plugins(DefaultPlugins);
	}

	app.insert_resource(ClearColor(Color::WHITE * 0.9))
		.add_plugin(HanabiPlugin)
		.insert_resource(RapierConfiguration {
			gravity: -Vec2::Y * 80.0,
			..default()
		})
		.add_plugin(WorldInspectorPlugin::new())
		.add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0))
		.add_plugin(input::InputPlugin)
		.add_plugin(states::StatePlugin)
		.add_plugin(player::PlayerPlugin)
		.add_plugin(level::LevelPlugin)
		.add_plugin(menu::MenuPlugin)
		.add_plugin(game::GamePlugin)
		.add_plugin(editor::EditorPlugin)
		.add_plugin(leaderboard::LeaderboardPlugin);

	if DEBUG_SCHEDULE {
		bevy_mod_debugdump::print_main_schedule(&mut app);
	}

	app.run();
}
