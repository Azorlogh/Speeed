use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_ecs_ldtk::{ldtk, LdtkWorldBundle, LevelSet};
use bevy_rapier2d::prelude::*;

use crate::{
	player,
	states::{AppState, Exit},
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			(setup, crate::level::spawn_wall_collision).in_schedule(OnEnter(AppState::Game)),
		)
		.add_system(exit.in_schedule(OnExit(AppState::Game)))
		.add_system(back_to_menu)
		.add_systems((spawn_player,).distributive_run_if(in_state(AppState::Game)));
	}
}

fn back_to_menu(mut next_app_state: ResMut<NextState<AppState>>, keys: Res<Input<KeyCode>>) {
	if keys.just_pressed(KeyCode::Escape) {
		next_app_state.set(AppState::Menu);
	}
}

fn exit(mut commands: Commands) {}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	let mut camera = Camera2dBundle::default();
	camera.projection.scaling_mode = ScalingMode::FixedVertical(2.0);
	camera.projection.scale = 2f32.powf(3.0);
	commands.spawn(camera);

	let ldtk_handle = asset_server.load("levels.ldtk");
	commands.spawn((
		LdtkWorldBundle {
			ldtk_handle,
			transform: Transform::from_scale(Vec3::splat(1.0 / 16.0)),
			..default()
		},
		Exit(AppState::Game),
	));
}

fn spawn_player(
	mut commands: Commands,
	q_spawned_ldtk_entities: Query<&ldtk::EntityInstance, Added<ldtk::EntityInstance>>,
	// q_spawn: Query<&ldtk::EntityInstance, Added<level::Spawn>>,
	q_camera: Query<Entity, With<Camera>>,
) {
	for spawn in q_spawned_ldtk_entities
		.iter()
		.filter(|e| e.identifier == "Start")
	{
		let cam_entity = q_camera.single();

		commands
			.spawn((
				player::PlayerBundle {
					spatial: SpatialBundle::from_transform(Transform::from_translation(
						(spawn.grid.as_vec2() + 0.5).extend(0.0),
					)),
					..default()
				},
				Exit(AppState::Game),
			))
			.add_child(cam_entity);
	}
}
