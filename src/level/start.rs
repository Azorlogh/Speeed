use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
	game::grid_to_world,
	player,
	states::{AppState, Exit},
};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Start;

pub fn spawn_start(
	mut commands: Commands,
	q_layer: Query<&LayerMetadata>,
	q_spawned_ldtk_entities: Query<(Entity, &ldtk::EntityInstance), Added<ldtk::EntityInstance>>,
	q_camera: Query<Entity, With<Camera>>,
) {
	for (entity, spawn) in q_spawned_ldtk_entities
		.iter()
		.filter(|(_, e)| e.identifier == "Start")
	{
		let cam_entity = q_camera.single();

		commands.entity(entity).insert(Start);

		commands
			.spawn((
				player::PlayerBundle {
					spatial: SpatialBundle::from_transform(Transform::from_translation(
						grid_to_world(q_layer.single(), spawn.grid).extend(0.0),
					)),
					..default()
				},
				Exit(AppState::Game),
			))
			.add_child(cam_entity);
	}
}
