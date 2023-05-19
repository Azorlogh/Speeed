use std::error::Error;

use bevy::prelude::*;
use bevy_ecs_ldtk::{
	ldtk::{self, ldtk_fields::LdtkFields},
	LayerMetadata,
};

use crate::{
	game::grid_to_world,
	states::{AppState, Exit},
};

pub struct TextPlugin;

impl Plugin for TextPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_system(spawn_text.run_if(in_state(AppState::Game)));
	}
}

fn spawn_text(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	q_layer: Query<&LayerMetadata>,
	q_spawned_ldtk_entities: Query<&ldtk::EntityInstance, Added<ldtk::EntityInstance>>,
) {
	for instance in q_spawned_ldtk_entities
		.iter()
		.filter(|e| e.identifier == "Text")
	{
		if let Err(e) = (|| {
			let content = instance
				.get_string_field("content")?
				.replace("<jump>", "Space");

			commands.spawn((
				Text2dBundle {
					text: Text::from_section(
						content,
						TextStyle {
							font: asset_server.load("fonts/FiraSans-Bold.ttf"),
							font_size: 50.0,
							color: Color::BLACK,
							..default()
						},
					)
					.with_alignment(TextAlignment::Center),
					transform: Transform::from_translation(
						grid_to_world(q_layer.single(), instance.grid).extend(0.0),
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
