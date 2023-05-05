use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use states::AppState;

mod editor;
mod game;
mod menu;
mod player;
mod states;

fn main() {
	App::new()
		.insert_resource(ClearColor(Color::WHITE * 0.9))
		.add_plugins(DefaultPlugins)
		.insert_resource(RapierConfiguration {
			gravity: -Vec2::Y * 2000.0,
			..default()
		})
		.add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(500.0))
		.add_state::<AppState>()
		.add_plugin(player::PlayerPlugin)
		.add_plugin(menu::MenuPlugin)
		.add_plugin(game::GamePlugin)
		.add_plugin(editor::EditorPlugin)
		.run();
}
