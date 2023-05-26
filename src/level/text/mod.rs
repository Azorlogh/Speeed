use std::error::Error;

use bevy::prelude::*;
use bevy_ecs_ldtk::ldtk::{self, ldtk_fields::LdtkFields};

use super::LevelSize;
use crate::{
	game::grid_to_world,
	input::CurrentInputMode,
	states::{AppState, Exit},
};

pub struct TextPlugin;

impl Plugin for TextPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_systems((spawn_text, update_text).distributive_run_if(in_state(AppState::Game)));
	}
}

#[derive(Component)]
pub struct FloatingText(String);

fn spawn_text(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	level_size: LevelSize,
	q_spawned_ldtk_entities: Query<&ldtk::EntityInstance, Added<ldtk::EntityInstance>>,
) {
	for instance in q_spawned_ldtk_entities
		.iter()
		.filter(|e| e.identifier == "Text")
	{
		if let Err(e) = (|| {
			let source = instance.get_string_field("content")?;

			commands.spawn((
				FloatingText(source.clone()),
				Text2dBundle {
					text: Text::from_section(
						display_for_keyboard(&source),
						TextStyle {
							font: asset_server.load("fonts/FiraSans-Bold.ttf"),
							font_size: 50.0,
							color: Color::WHITE,
							..default()
						},
					)
					.with_alignment(TextAlignment::Center),
					transform: Transform::from_translation(
						grid_to_world(&level_size, instance.grid).extend(0.0),
					)
					.with_scale(Vec3::splat(1.0 / 50.0 * 0.6)),
					..default()
				},
				Exit(AppState::Game),
			));

			Result::<_, Box<dyn Error>>::Ok(())
		})() {
			warn!("failed to spawn launchpad: {e}");
		}
	}
}

fn update_text(input_mode: Res<CurrentInputMode>, mut q_text: Query<(&mut Text, &FloatingText)>) {
	if input_mode.is_changed() {
		for (mut text, src) in &mut q_text {
			match input_mode.as_ref() {
				CurrentInputMode::Keyboard => text.sections[0].value = display_for_keyboard(&src.0),
				CurrentInputMode::Gamepad => text.sections[0].value = display_for_gamepad(&src.0),
			}
		}
	}
}

fn display_for_keyboard(input: &str) -> String {
	input
		.replace("<jump>", "Space")
		.replace("<ground_pound>", "Down Arrow")
}

fn display_for_gamepad(input: &str) -> String {
	input.replace("<jump>", "A").replace("<ground_pound>", "X")
}
