use bevy::{
	input::gamepad::{GamepadConnection, GamepadEvent},
	prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

mod editor;
mod game;
mod level;
mod menu;
mod player;
mod states;

fn main() {
	App::new()
		.insert_resource(ClearColor(Color::WHITE * 0.9))
		.add_plugins(DefaultPlugins)
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
		// .add_system(gamepad_connections)
		.run();
}

// #[derive(Resource)]
// struct MyGamepad(Gamepad);

// fn gamepad_connections(
// 	mut commands: Commands,
// 	my_gamepad: Option<Res<MyGamepad>>,
// 	mut gamepad_evr: EventReader<GamepadEvent>,
// ) {
// 	for ev in gamepad_evr.iter().filter_map(|evt| match evt {
// 		GamepadEvent::Connection(e) => Some(e),
// 		_ => None,
// 	}) {
// 		let id = ev.gamepad;
// 		match &ev.connection {
// 			GamepadConnection::Connected(info) => {
// 				println!(
// 					"New gamepad connected with ID: {:?}, name: {}",
// 					id, info.name
// 				);
// 				if my_gamepad.is_none() {
// 					commands.insert_resource(MyGamepad(id));
// 				}
// 			}
// 			GamepadConnection::Disconnected => {
// 				println!("Lost gamepad connection with ID: {:?}", id);
// 				if let Some(MyGamepad(old_id)) = my_gamepad.as_deref() {
// 					if *old_id == id {
// 						commands.remove_resource::<MyGamepad>();
// 					}
// 				}
// 			}
// 		}
// 	}
// }
