use bevy::{prelude::*, render::texture::DEFAULT_IMAGE_HANDLE};
use bevy_rapier2d::prelude::*;

use crate::{
	game::Restart,
	input::{self, Action},
	level::RestoresJump,
	states::{AppState, Exit},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<SpawnPlayer>().add_systems(
			(
				player_spawn,
				player_controls,
				player_jumps,
				player_render,
				player_restart,
			)
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
	ground_pound: bool,
	jumping: bool,
	in_air: bool,
}

const PLAYER_SIZE: f32 = 0.5;
const PLAYER_SPEED: f32 = 150.0;
const PLAYER_SPEED_IN_AIR: f32 = 13.0;
const PLAYER_FRICTION: f32 = 0.1;

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
				ActiveEvents::COLLISION_EVENTS | ActiveEvents::CONTACT_FORCE_EVENTS,
				Exit(AppState::Game),
			))
			.id();

		// Player
		commands
			.spawn((
				(
					Player {
						jump_vel: 20.0,
						speed: PLAYER_SPEED,
						remaining_jumps: 1,
						ground_pound: false,
						jumping: false,
						in_air: true,
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
					linear_damping: 2.0,
					angular_damping: 0.0,
				},
				GravityScale(1.0),
				Friction {
					coefficient: PLAYER_FRICTION,
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
		player.jumping = true;
		player.in_air = true;
		gravity.0 = 0.5;
	}
	if action.just_released(Action::Jump) {
		player.jumping = false;
		gravity.0 = 1.0;
	}

	if action.just_pressed(Action::GroundPound) {
		velocity.linvel.y = -player.jump_vel * 2.0;
	}

	ext_force.force = Vec2::ZERO;
	let speed = if player.in_air {
		PLAYER_SPEED_IN_AIR
	} else {
		PLAYER_SPEED
	};
	// let speed = if player.in_air { 0.1 } else { 1.0 };
	if action.pressed(Action::Left) {
		ext_force.force += Vec2::new(-speed, 0.0);
	}
	if action.pressed(Action::Right) {
		ext_force.force += Vec2::new(speed, 0.0);
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
	mut ev_collision: EventReader<CollisionEvent>,
	mut ev_contact_forces: EventReader<ContactForceEvent>,
	mut q_player: Query<(Entity, &mut Player, &mut GravityScale, &Velocity)>,
	mut q_walljump_sensor: Query<Entity, With<PlayerWalljumpSensor>>,
	q_wall: Query<Option<&RestoresJump>>,
) {
	let Ok((player_entity, mut player, mut gravity, velocity)) = q_player.get_single_mut() else {
		return;
	};
	let Ok(walljump_sensor_entity) = q_walljump_sensor.get_single_mut() else {
		return;
	};

	for collision_event in ev_collision.iter() {
		match collision_event {
			CollisionEvent::Started(e0, e1, _) => {
				let wall_entity = match walljump_sensor_entity {
					e if *e0 == e => e1,
					e if *e1 == e => e0,
					_ => break,
				};
				let restores_jump = q_wall.get(*wall_entity).unwrap();
				if restores_jump.is_some() {
					player.remaining_jumps = 1;
				}
			}
			_ => {}
		}
	}
	for forces in ev_contact_forces.iter() {
		if forces.collider1 == player_entity || forces.collider2 == player_entity {
			if forces.max_force_direction.dot(Vec2::Y).abs() > 0.7 {
				player.ground_pound = false;
				player.in_air = false;
			}
		}
		// match contact_forces {
		// 	ContactForcesEvent::Started(e0, e1, _) if *e0 == e || *e1 == e => {
		// 		let wall_entity = match walljump_sensor_entity {
		// 			e if * == e => e1,
		// 			e if *e1 == e => e0,
		// 			_ => break,
		// 		};
		// 		let restores_jump = q_wall.get(*wall_entity).unwrap();
		// 		if restores_jump.is_some() {
		// 			player.remaining_jumps = 1;
		// 		}
		// 	}
		// 	_ => {}
		// }
	}
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
