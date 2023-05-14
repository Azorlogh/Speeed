use std::time::Instant;

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_ecs_ldtk::{ldtk, LayerMetadata, LdtkWorldBundle};
use bevy_hanabi::prelude::*;
use bevy_rapier2d::prelude::CollisionEvent;

use crate::{
	input::Action,
	leaderboard::{CurrentScore, Leaderboard},
	level::{
		finish::{self, Finish},
		Spawn,
	},
	player::{self, Player},
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
		.add_systems(
			(spawn_player, spawn_finish, restart).distributive_run_if(in_state(AppState::Game)),
		)
		.add_system(finish.run_if(in_state(AppState::Game)));
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.insert_resource(StartTime(Instant::now()));

	let mut camera = Camera2dBundle::default();
	camera.projection.scaling_mode = ScalingMode::FixedVertical(2.0);
	camera.projection.scale = 2f32.powf(3.0);
	commands.spawn((camera, Exit(AppState::Game)));

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
	q_layer: Query<&LayerMetadata>,
	q_spawned_ldtk_entities: Query<(Entity, &ldtk::EntityInstance), Added<ldtk::EntityInstance>>,
	q_camera: Query<Entity, With<Camera>>,
) {
	for (entity, spawn) in q_spawned_ldtk_entities
		.iter()
		.filter(|(_, e)| e.identifier == "Start")
	{
		let cam_entity = q_camera.single();

		commands.entity(entity).insert(Spawn);

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

fn spawn_finish(
	mut commands: Commands,
	mut effects: ResMut<Assets<EffectAsset>>,
	q_layer: Query<&LayerMetadata>,
	q_spawned_ldtk_entities: Query<&ldtk::EntityInstance, Added<ldtk::EntityInstance>>,
) {
	for finish in q_spawned_ldtk_entities
		.iter()
		.filter(|e| e.identifier == "Finish")
	{
		let mut gradient = Gradient::new();
		gradient.add_key(0.0, Vec4::new(0.5, 0.5, 1.0, 1.0));
		gradient.add_key(1.0, Vec4::new(0.5, 0.5, 1.0, 0.0));

		let spawner = Spawner::rate(30.0.into());
		let effect = effects.add(
			EffectAsset {
				name: "FinishEffect".into(),
				capacity: 4096,
				spawner,
				..Default::default()
			}
			.init(InitPositionCircleModifier {
				center: Vec3::ZERO,
				axis: Vec3::Z,
				radius: 2.0,
				dimension: ShapeDimension::Surface,
			})
			.init(InitVelocityCircleModifier {
				center: Vec3::ZERO,
				axis: Vec3::Z,
				speed: (-0.3f32).into(),
			})
			.init(InitLifetimeModifier {
				lifetime: 5_f32.into(),
			})
			.render(SizeOverLifetimeModifier {
				gradient: Gradient::constant(Vec2::splat(0.02)),
			})
			.render(ColorOverLifetimeModifier { gradient }),
		);

		commands
			.spawn(ParticleEffectBundle {
				effect: ParticleEffect::new(effect).with_z_layer_2d(Some(0.1)),
				..default()
			})
			.insert((
				finish::FinishBundle {
					spatial: SpatialBundle::from_transform(Transform::from_translation(
						grid_to_world(q_layer.single(), finish.grid).extend(0.0),
					)),
					..default()
				},
				Exit(AppState::Game),
			));
	}
}

pub fn grid_to_world(layer: &LayerMetadata, coord: IVec2) -> Vec2 {
	Vec2::new(
		coord.x as f32 + 0.5,
		layer.c_hei as f32 - coord.y as f32 + 0.5,
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
					let score = start_time.0.elapsed().as_millis() as u64;
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
	mut q_player: Query<&mut Transform, With<Player>>,
	mut q_spawn: Query<&ldtk::EntityInstance, With<Spawn>>,
	mut next_state: ResMut<NextState<AppState>>,
) {
	if actions.just_pressed(Action::Restart) {
		next_state.set(AppState::Game);
	}
}
