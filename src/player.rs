use bevy::{prelude::*, render::texture::DEFAULT_IMAGE_HANDLE};
use bevy_rapier2d::prelude::*;

use crate::states::AppState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(Input::<Action>::default())
			.add_system(handle_inputs)
			.add_systems(
				(player_controls, player_jumps, player_render)
					.after(handle_inputs)
					.distributive_run_if(in_state(AppState::Game)),
			);
	}
}

#[derive(Clone, Debug, Default)]
pub struct GamepadAxes {
	horizontal: f32,
}
fn handle_inputs(
	mut actions: ResMut<Input<Action>>,
	gamepad_axes: Res<Axis<GamepadAxis>>,
	buttons: Res<Input<GamepadButton>>,
	gamepads: Res<Gamepads>,
	keys: Res<Input<KeyCode>>,
	mut prev_axes: Local<GamepadAxes>,
) {
	let gamepad = gamepads.iter().next();
	actions.clear();
	let mut axes = GamepadAxes::default();
	if let Some(gp) = gamepad {
		axes.horizontal = gamepad_axes
			.get(GamepadAxis {
				gamepad: gp,
				axis_type: GamepadAxisType::LeftStickX,
			})
			.unwrap();
	}
	dbg!(&axes);
	if keys.just_pressed(KeyCode::Space)
		|| gamepad
			.map(|gp| {
				buttons.just_pressed(GamepadButton {
					gamepad: gp,
					button_type: GamepadButtonType::South,
				})
			})
			.unwrap_or(false)
	{
		actions.press(Action::Jump);
	}
	if keys.just_released(KeyCode::Space)
		|| gamepad
			.map(|gp| {
				buttons.just_released(GamepadButton {
					gamepad: gp,
					button_type: GamepadButtonType::South,
				})
			})
			.unwrap_or(false)
	{
		actions.release(Action::Jump);
	}
	const STICK_THESHOLD: f32 = 0.8;
	if keys.just_pressed(KeyCode::Left)
		|| (prev_axes.horizontal >= -STICK_THESHOLD && axes.horizontal < -STICK_THESHOLD)
	{
		actions.press(Action::Left);
	}
	if keys.just_released(KeyCode::Left)
		|| (prev_axes.horizontal < -STICK_THESHOLD && axes.horizontal >= -STICK_THESHOLD)
	{
		actions.release(Action::Left);
	}
	if keys.just_pressed(KeyCode::Right)
		|| (prev_axes.horizontal <= STICK_THESHOLD && axes.horizontal > STICK_THESHOLD)
	{
		actions.press(Action::Right);
	}
	if keys.just_released(KeyCode::Right)
		|| (prev_axes.horizontal > STICK_THESHOLD && axes.horizontal <= STICK_THESHOLD)
	{
		actions.release(Action::Right);
	}
	*prev_axes = axes.clone();
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Action {
	Jump,
	Left,
	Right,
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

#[derive(Bundle)]
pub struct PlayerBundle {
	pub sprite: Sprite,
	pub spatial: SpatialBundle,
	pub texture: Handle<Image>,
	pub rigid_body: RigidBody,
	pub velocity: Velocity,
	pub collider: Collider,
	pub external_force: ExternalForce,
	pub external_impulse: ExternalImpulse,
	pub player: Player,
	pub locked_axes: LockedAxes,
	pub damping: Damping,
	pub active_events: ActiveEvents,
	pub gravity_scale: GravityScale,
	pub friction: Friction,
	pub restitution: Restitution,
}
impl Default for PlayerBundle {
	fn default() -> Self {
		Self {
			player: Player {
				jump_vel: 30.0,
				speed: 15.0,
				remaining_jumps: 1,
				in_air: false,
				jumping: false,
			},
			sprite: Sprite {
				color: Color::rgb(0.25, 0.25, 0.75),
				custom_size: Some(Vec2::splat(PLAYER_SIZE)),
				..default()
			},
			spatial: SpatialBundle::default(),
			texture: DEFAULT_IMAGE_HANDLE.typed(),
			rigid_body: RigidBody::Dynamic,
			velocity: Velocity::zero(),
			collider: Collider::ball(PLAYER_SIZE / 2.0),
			external_force: ExternalForce::default(),
			external_impulse: ExternalImpulse::default(),
			locked_axes: LockedAxes::ROTATION_LOCKED,
			damping: Damping {
				linear_damping: 5.0,
				angular_damping: 0.0,
			},
			active_events: ActiveEvents::COLLISION_EVENTS,
			gravity_scale: GravityScale(1.0),
			friction: Friction {
				coefficient: 0.0,
				combine_rule: CoefficientCombineRule::Min,
			},
			restitution: Restitution::default(),
		}
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
		// ext_impulse.impulse += Vec2::Y * player.jump_force;
		velocity.linvel.y = player.jump_vel;
		player.remaining_jumps -= 1;
		player.in_air = true;
		player.jumping = true;
		gravity.0 = 0.25;
	}
	if action.just_pressed(Action::Jump) {
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
		sprite.color = Color::BLACK;
	} else if player.jumping {
		sprite.color = Color::GREEN;
	} else {
		sprite.color = Color::RED;
	}
}

fn player_jumps(
	mut collision_events: EventReader<CollisionEvent>,
	mut q_player: Query<(Entity, &mut Player, &mut GravityScale, &Velocity)>,
) {
	let Ok((entity, mut player, mut gravity, velocity)) = q_player.get_single_mut() else {
		return;
	};
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
	if player.jumping && velocity.linvel.y < 0.0 {
		player.jumping = false;
		gravity.0 = 1.0;
	}
}
