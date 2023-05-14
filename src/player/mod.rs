use bevy::{prelude::*, render::texture::DEFAULT_IMAGE_HANDLE};
use bevy_rapier2d::prelude::*;

use crate::{
	input::{self, Action},
	states::{AppState, Exit},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<SpawnPlayer>().add_systems(
			(player_spawn, player_controls, player_jumps, player_render)
				.after(input::handle_inputs)
				.distributive_run_if(in_state(AppState::Game)),
		);
	}
}

#[derive(Component)]
pub struct Player {
	jump_vel: f32,
	speed: f32,
	remaining_jumps: usize,
	in_air: bool,
	jumping: bool,
}

const PLAYER_SIZE: f32 = 0.5;

#[derive(Component)]
pub struct PlayerWalljumpSensor;

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
				Exit(AppState::Game),
			))
			.id();

		// Player
		commands
			.spawn((
				(
					Player {
						jump_vel: 30.0,
						speed: 15.0,
						remaining_jumps: 1,
						in_air: false,
						jumping: false,
					},
					Sprite {
						color: Color::rgb(0.25, 0.25, 0.75),
						custom_size: Some(Vec2::splat(PLAYER_SIZE)),
						..default()
					},
					SpatialBundle::from_transform(Transform::from_translation(pos.extend(1.0))),
					DEFAULT_IMAGE_HANDLE.typed::<Image>(),
					RigidBody::Dynamic,
					Velocity::zero(),
					Collider::ball(PLAYER_SIZE / 2.0),
					ExternalForce::default(),
					ExternalImpulse::default(),
					LockedAxes::ROTATION_LOCKED,
				),
				Damping {
					linear_damping: 5.0,
					angular_damping: 0.0,
				},
				GravityScale(1.0),
				Friction {
					coefficient: 0.0,
					combine_rule: CoefficientCombineRule::Min,
				},
				Restitution::default(),
				ActiveEvents::COLLISION_EVENTS,
				Exit(AppState::Game),
			))
			.add_child(cam_entity)
			.add_child(walljump_sensor);
	}
}

fn player_controls(
	action: Res<Input<Action>>,
	mut q_player: Query<(
		&mut Player,
		&mut ExternalForce,
		&mut GravityScale,
		&mut Velocity,
	)>,
) {
	let Ok((mut player, mut ext_force, mut gravity, mut velocity)) = q_player.get_single_mut() else {
		return;
	};

	if action.just_pressed(Action::Jump) && player.remaining_jumps > 0 {
		velocity.linvel.y = player.jump_vel;
		player.remaining_jumps -= 1;
		player.in_air = true;
		player.jumping = true;
		gravity.0 = 0.5;
	}
	if action.just_released(Action::Jump) {
		player.jumping = false;
		gravity.0 = 1.0;
	}
	ext_force.force = Vec2::ZERO;
	if action.pressed(Action::Left) {
		ext_force.force += Vec2::new(-player.speed, 0.0);
	}
	if action.pressed(Action::Right) {
		ext_force.force += Vec2::new(player.speed, 0.0);
	}
}

fn player_render(mut q_player: Query<(&Player, &mut Sprite)>) {
	let Ok((player, mut sprite)) = q_player.get_single_mut() else {
		return;
	};
	if player.remaining_jumps != 0 {
		sprite.color = Color::RED;
	} else if player.jumping {
		sprite.color = Color::BLACK;
	}
}

fn player_jumps(
	mut collision_events: EventReader<CollisionEvent>,
	mut q_player: Query<(Entity, &mut Player, &mut GravityScale, &Velocity)>,
	mut q_walljump_sensor: Query<Entity, With<PlayerWalljumpSensor>>,
) {
	let Ok((_, mut player, mut gravity, velocity)) = q_player.get_single_mut() else {
		return;
	};
	let Ok(walljump_sensor_entity) = q_walljump_sensor.get_single_mut() else {
		return;
	};
	for collision_event in collision_events.iter() {
		match collision_event {
			CollisionEvent::Started(e0, e1, _) => {
				if *e0 == walljump_sensor_entity || *e1 == walljump_sensor_entity {
					player.remaining_jumps = 1;
					player.in_air = false;
				}
			}
			_ => {}
		}
	}
	if player.jumping && velocity.linvel.y < 0.0 {
		player.jumping = false;
		gravity.0 = 1.0;
	}
}
