use bevy::{prelude::*, render::texture::DEFAULT_IMAGE_HANDLE};
use bevy_rapier2d::prelude::*;

use crate::{
	game::Restart,
	input::{self, Action},
	level::{RestoresJump, WallCollider},
	states::{AppState, Exit},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<Player>()
			.add_event::<SpawnPlayer>()
			.add_systems(
				(
					player_spawn,
					player_on_ground,
					player_jumps,
					player_controls.after(player_on_ground).after(player_jumps),
					player_render,
					player_restart,
				)
					.after(input::handle_inputs)
					.distributive_run_if(in_state(AppState::Game)),
			);
	}
}

#[derive(Component, Reflect)]
pub struct Player {
	pub jump_vel: f32,
	pub speed: f32,
	pub remaining_jumps: usize,
	pub ground_pound: bool,
	pub jumping: bool,
	pub in_air: bool,
	pub on_wall: bool,
	pub swapped: bool, // whether the left&right direction are swapped (useful for portals)
}

const PLAYER_SIZE: f32 = 0.5;
const PLAYER_MAX_SPEED: f32 = 12.0;

#[derive(Component)]
pub struct PlayerWalljumpSensor;
#[derive(Component)]
pub struct PlayerGroundSensor;

pub struct SpawnPlayer {
	pub pos: Vec2,
}
fn player_spawn(
	mut commands: Commands,
	mut ev_spawn_player: EventReader<SpawnPlayer>,
	q_camera: Query<Entity, With<Camera>>,
) {
	if let Some(SpawnPlayer { pos }) = ev_spawn_player.iter().last() {
		let cam_entity = q_camera.single();

		// Sensor for detecting walls (rectangle with the sides sticking out)
		let walljump_sensor = commands
			.spawn((
				PlayerWalljumpSensor,
				Collider::cuboid(PLAYER_SIZE / 2.0 * 1.4, PLAYER_SIZE / 2.0 * 1.0),
				ColliderMassProperties::Density(0.0),
				Sensor,
				TransformBundle::from(Transform::from_translation(Vec3::new(
					0.0,
					-PLAYER_SIZE * 0.2,
					0.0,
				))),
				ActiveEvents::COLLISION_EVENTS,
				CollidingEntities::default(),
				Exit(AppState::Game),
			))
			.id();

		// Sensor for detecting walls (rectangle with the sides sticking out)
		let ground_sensor = commands
			.spawn((
				PlayerGroundSensor,
				Collider::cuboid(PLAYER_SIZE / 2.0 * 0.8, PLAYER_SIZE / 2.0 * 1.0),
				ColliderMassProperties::Density(0.0),
				Sensor,
				TransformBundle::from(Transform::from_translation(Vec3::new(
					0.0,
					-PLAYER_SIZE * 0.2,
					0.0,
				))),
				ActiveEvents::COLLISION_EVENTS,
				CollidingEntities::default(),
				Exit(AppState::Game),
			))
			.id();

		// Player
		commands
			.spawn((
				Player {
					jump_vel: 23.1,
					speed: 50.0,
					remaining_jumps: 1,
					ground_pound: false,
					jumping: false,
					in_air: true,
					on_wall: false,
					swapped: false,
				},
				Sprite {
					color: Color::rgb(0.25, 0.25, 0.75),
					custom_size: Some(Vec2::splat(PLAYER_SIZE)),
					..default()
				},
				SpatialBundle::from_transform(Transform::from_translation(pos.extend(1.0))),
				DEFAULT_IMAGE_HANDLE.typed::<Image>(),
				(
					RigidBody::Dynamic,
					Velocity::zero(),
					Collider::ball(PLAYER_SIZE / 2.0),
					ExternalForce::default(),
					ExternalImpulse::default(),
					LockedAxes::ROTATION_LOCKED,
					ColliderMassProperties::Density(10.0),
					Damping {
						linear_damping: 2.0,
						angular_damping: 0.0,
					},
					GravityScale(1.0),
					Friction {
						coefficient: 0.0,
						combine_rule: CoefficientCombineRule::Min,
					},
					CollidingEntities::default(),
					Restitution::default(),
					ActiveEvents::COLLISION_EVENTS,
					Ccd::enabled(),
				),
				Exit(AppState::Game),
			))
			.add_child(cam_entity)
			.add_child(walljump_sensor)
			.add_child(ground_sensor);
	}
}

pub fn player_controls(
	action: Res<Input<Action>>,
	time: Res<Time>,
	mut q_player: Query<(
		&mut Player,
		&mut ExternalForce,
		&mut GravityScale,
		&mut Velocity,
		&mut Friction,
	)>,
) {
	let Ok((mut player, mut ext_force, mut gravity, mut velocity, mut friction)) = q_player.get_single_mut() else {
		return;
	};

	if action.just_pressed(Action::Jump) && player.remaining_jumps > 0 {
		velocity.linvel.y = velocity.linvel.y.max(player.jump_vel);
		player.remaining_jumps -= 1;
		player.jumping = true;
		gravity.0 = 0.5;
	}
	if action.just_released(Action::Jump) {
		player.jumping = false;
		gravity.0 = 1.0;
	}

	if action.just_pressed(Action::GroundPound) {
		velocity.linvel.y = -player.jump_vel * 2.0;
		player.ground_pound = true;
	}

	ext_force.force = Vec2::ZERO;
	if (action.pressed(Action::Left) && !player.swapped
		|| action.pressed(Action::Right) && player.swapped)
		&& velocity.linvel.x > -PLAYER_MAX_SPEED
	{
		if velocity.linvel.x > -PLAYER_MAX_SPEED {
			velocity.linvel.x =
				(velocity.linvel.x - player.speed * time.delta_seconds()).max(-PLAYER_MAX_SPEED);
		}
	}
	if (action.pressed(Action::Right) && !player.swapped
		|| action.pressed(Action::Left) && player.swapped)
		&& velocity.linvel.x < PLAYER_MAX_SPEED
	{
		if velocity.linvel.x < PLAYER_MAX_SPEED {
			velocity.linvel.x =
				(velocity.linvel.x + player.speed * time.delta_seconds()).min(PLAYER_MAX_SPEED);
		}
	}

	if action.just_released(Action::Left) || action.just_released(Action::Right) {
		player.swapped = false;
	}

	if !action.pressed(Action::Left) && !action.pressed(Action::Right) {
		// prevent sliding when the user is not moving sideways
		friction.coefficient = 1.0;
	} else {
		friction.coefficient = 0.0;
	}
}

fn player_render(mut q_player: Query<(&Player, &mut Sprite)>) {
	let Ok((player, mut sprite)) = q_player.get_single_mut() else {
		return;
	};
	if player.remaining_jumps != 0 {
		sprite.color = Color::RED * 0.1;
	} else {
		sprite.color = Color::BLACK;
	}
}

fn player_on_ground(
	mut q_player: Query<&mut Player>,
	q_sensor: Query<&CollidingEntities, With<PlayerGroundSensor>>,
	q_wall: Query<Entity, With<WallCollider>>,
) {
	let Ok(mut player) = q_player.get_single_mut() else {
		return;
	};

	let Ok(ground_sensor) = q_sensor.get_single() else {
		return;
	};

	player.in_air = !ground_sensor.iter().any(|e| q_wall.get(e).is_ok());
	if !player.in_air {
		player.remaining_jumps = 1;
	}
}

fn player_jumps(
	mut ev_collision: EventReader<CollisionEvent>,
	mut q_player: Query<(&mut Player, &mut GravityScale, &Velocity)>,
	mut q_walljump_sensor: Query<(Entity, &CollidingEntities), With<PlayerWalljumpSensor>>,
	q_wall: Query<Option<&RestoresJump>>,
) {
	let Ok((mut player, mut gravity, velocity)) = q_player.get_single_mut() else {
		return;
	};
	let Ok((walljump_sensor_entity, walljump_colliding_entities)) = q_walljump_sensor.get_single_mut() else {
		return;
	};

	if !player.on_wall {
		for collision_event in ev_collision.iter() {
			match collision_event {
				CollisionEvent::Started(e0, e1, _) => {
					let wall_entity = match walljump_sensor_entity {
						e if *e0 == e => e1,
						e if *e1 == e => e0,
						_ => continue,
					};
					let restores_jump = q_wall.get(*wall_entity).unwrap();
					if restores_jump.is_some() {
						player.remaining_jumps = 1;
					}
				}
				_ => {}
			}
		}
	}

	player.on_wall = !walljump_colliding_entities.is_empty();

	if player.jumping && velocity.linvel.y < 0.0 {
		player.jumping = false;
		gravity.0 = 1.0;
	}
}

fn player_restart(mut ev_restart: EventWriter<Restart>, q_player: Query<&Transform, With<Player>>) {
	let Ok(transform) = q_player.get_single() else {
		return;
	};
	if transform.translation.y <= -5.0 {
		ev_restart.send(Restart)
	}
}
