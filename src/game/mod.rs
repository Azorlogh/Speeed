use std::time::Instant;

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_ecs_ldtk::{LayerMetadata, LdtkWorldBundle, LevelSelection, LevelSet, Respawn};
use bevy_rapier2d::prelude::CollisionEvent;

use crate::{
	input::Action,
	leaderboard::{CurrentScore, Leaderboard, Score},
	level::finish::Finish,
	player::Player,
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
		.add_systems((restart, finish).distributive_run_if(in_state(AppState::Game)))
		.add_startup_system(setup_level);
	}
}

#[derive(Resource)]
pub struct StartTime(Instant);

fn back_to_menu(mut next_app_state: ResMut<NextState<AppState>>, keys: Res<Input<KeyCode>>) {
	if keys.just_pressed(KeyCode::Escape) {
		next_app_state.set(AppState::Menu);
	}
}

fn exit(mut _commands: Commands) {}

fn setup_level(mut commands: Commands, asset_server: Res<AssetServer>) {
	let ldtk_handle = asset_server.load("levels.ldtk");
	commands.spawn(LdtkWorldBundle {
		ldtk_handle,
		transform: Transform::from_scale(Vec3::splat(1.0 / 16.0)),
		..default()
	});
}

fn setup(
	mut commands: Commands,
	mut level_selection: ResMut<LevelSelection>,
	q_ldtk_world: Query<Entity, With<LevelSet>>,
) {
	commands.insert_resource(StartTime(Instant::now()));

	let mut camera = Camera2dBundle::default();
	camera.projection.scaling_mode = ScalingMode::FixedVertical(2.0);
	camera.projection.scale = 2f32.powf(3.0);
	camera.transform.translation.z -= 100.0;
	commands.spawn((camera, Exit(AppState::Game)));

	// let world = q_ldtk_world.single();
	// if *level_selection == LevelSelection::Index(0) {
	// 	commands.entity(world).insert(Respawn);
	// } else {
	// 	*level_selection = LevelSelection::Index(0);
	// }
}

pub fn grid_to_world(layer: &LayerMetadata, coord: IVec2) -> Vec2 {
	Vec2::new(
		coord.x as f32 + 0.5,
		layer.c_hei as f32 - coord.y as f32 - 0.5,
	)
}

fn finish(
	mut commands: Commands,
	mut collision_events: EventReader<CollisionEvent>,
	mut q_player: Query<Entity, With<Player>>,
	mut q_finish: Query<Entity, With<Finish>>,
	start_time: Res<StartTime>,
	mut leaderboard: ResMut<Leaderboard>,
	mut next_state: ResMut<NextState<AppState>>,
) {
	let Ok(player_entity) = q_player.get_single_mut() else {
		return;
	};
	let Ok(finish_entity) = q_finish.get_single_mut() else {
		return;
	};
	for collision_event in collision_events.iter() {
		match collision_event {
			CollisionEvent::Started(e0, e1, _) => {
				if (*e0 == player_entity && *e1 == finish_entity)
					|| (*e1 == player_entity && *e0 == finish_entity)
				{
					let score = Score(start_time.0.elapsed().as_millis() as u64);
					leaderboard.add_score(score);
					commands.insert_resource(CurrentScore(score));
					next_state.set(AppState::Leaderboard);
				}
			}
			_ => {}
		}
	}
}

fn restart(
	actions: Res<Input<Action>>,
	// mut q_player: Query<&mut Transform, With<Player>>,
	// mut q_spawn: Query<&ldtk::EntityInstance, With<Spawn>>,
	mut next_state: ResMut<NextState<AppState>>,
) {
	if actions.just_pressed(Action::Restart) {
		next_state.set(AppState::Game);
	}
}
