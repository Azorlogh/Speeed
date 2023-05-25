use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use super::LevelSize;
use crate::{game::grid_to_world, player::SpawnPlayer};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Start;

pub fn spawn_start(
	mut commands: Commands,
	level_size: LevelSize,
	q_spawned_ldtk_entities: Query<(Entity, &ldtk::EntityInstance), Added<ldtk::EntityInstance>>,
	mut ev_spawn_player: EventWriter<SpawnPlayer>,
) {
	for (entity, spawn) in q_spawned_ldtk_entities
		.iter()
		.filter(|(_, e)| e.identifier == "Start")
	{
		commands.entity(entity).insert(Start);

		ev_spawn_player.send(SpawnPlayer {
			pos: grid_to_world(&level_size, spawn.grid),
		});
	}
}
