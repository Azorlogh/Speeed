use bevy::prelude::*;
use bevy_hanabi::HanabiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

mod editor;
mod game;
mod leaderboard;
mod level;
mod menu;
mod player;
mod states;

fn main() {
	App::new()
		.insert_resource(ClearColor(Color::WHITE * 0.9))
		.add_plugins(DefaultPlugins)
		.add_plugin(HanabiPlugin)
		.insert_resource(RapierConfiguration {
			gravity: -Vec2::Y * 80.0,
			..default()
		})
		.add_plugin(WorldInspectorPlugin::new())
		.add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0))
		.add_plugin(states::StatePlugin)
		.add_plugin(player::PlayerPlugin)
		.add_plugin(level::LevelPlugin)
		.add_plugin(menu::MenuPlugin)
		.add_plugin(game::GamePlugin)
		.add_plugin(editor::EditorPlugin)
		.add_plugin(leaderboard::LeaderboardPlugin)
		.run();
}
