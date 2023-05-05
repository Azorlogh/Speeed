use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::states::AppState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_system(player_controls.run_if(in_state(AppState::Game)))
			.add_system(player_jumps.run_if(in_state(AppState::Game)))
			.add_system(player_render.run_if(in_state(AppState::Game)));
	}
}

#[derive(Component)]
pub struct Player {
	jump_force: f32,
	speed: f32,
	remaining_jumps: usize,
	in_air: bool,
}
impl Default for Player {
	fn default() -> Self {
		Self {
			jump_force: 5.0,
			speed: 10.0,
			remaining_jumps: 1,
			in_air: false,
		}
	}
}

#[derive(Bundle)]
pub struct PlayerBundle {
	sprite: SpriteBundle,
	rigid_body: RigidBody,
	velocity: Velocity,
	collider: Collider,
	external_force: ExternalForce,
	external_impulse: ExternalImpulse,
	player: Player,
	locked_axes: LockedAxes,
	damping: Damping,
	active_events: ActiveEvents,
	gravity_scale: GravityScale,
	friction: Friction,
}
impl Default for PlayerBundle {
	fn default() -> Self {
		Self {
			sprite: SpriteBundle {
				sprite: Sprite {
					color: Color::rgb(0.25, 0.25, 0.75),
					custom_size: Some(Vec2::new(50.0, 50.0)),
					..default()
				},
				transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
				..default()
			},
			rigid_body: RigidBody::Dynamic,
			velocity: Velocity::zero(),
			collider: Collider::ball(50.0 / 2.0),
			external_force: ExternalForce::default(),
			external_impulse: ExternalImpulse::default(),
			player: Player::default(),
			locked_axes: LockedAxes::ROTATION_LOCKED,
			damping: Damping {
				linear_damping: 5.0,
				angular_damping: 0.0,
			},
			active_events: ActiveEvents::COLLISION_EVENTS,
			gravity_scale: GravityScale(1.0),
			friction: Friction {
				coefficient: 0.0,
				combine_rule: default(),
			},
		}
	}
}

fn player_controls(
	keys: Res<Input<KeyCode>>,
	mut q_player: Query<(
		&mut Player,
		&mut ExternalForce,
		&mut ExternalImpulse,
		&mut GravityScale,
	)>,
) {
	let (mut player, mut ext_force, mut ext_impulse, mut gravity) = q_player.single_mut();
	if keys.just_pressed(KeyCode::Space) && player.remaining_jumps > 0 {
		ext_impulse.impulse += Vec2::Y * player.jump_force;
		player.remaining_jumps -= 1;
		player.in_air = true;
		gravity.0 = 0.5;
	}
	if keys.just_released(KeyCode::Space) {
		gravity.0 = 1.0;
	}
	ext_force.force = Vec2::ZERO;
	if keys.pressed(KeyCode::Left) {
		ext_force.force += Vec2::new(-player.speed, 0.0);
	}
	if keys.pressed(KeyCode::Right) {
		ext_force.force += Vec2::new(player.speed, 0.0);
	}
}

fn player_render(mut q_player: Query<(&Player, &mut Sprite)>) {
	let (player, mut sprite) = q_player.single_mut();
	if player.remaining_jumps != 0 {
		sprite.color = Color::BLACK;
	} else {
		sprite.color = Color::RED;
	}
}

fn player_jumps(
	mut collision_events: EventReader<CollisionEvent>,
	mut q_player: Query<(Entity, &mut Player)>,
) {
	let (entity, mut player) = q_player.single_mut();
	for collision_event in collision_events.iter() {
		match collision_event {
			CollisionEvent::Started(e0, e1, _) => {
				if *e0 == entity || *e1 == entity {
					player.remaining_jumps = 1;
					player.in_air = false;
				}
			}
			_ => {}
		}
	}
}
