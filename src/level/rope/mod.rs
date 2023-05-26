/// Rope element
/// Composed of a chain of small rigid-bodies
use std::error::Error;

use bevy::{prelude::*, render::texture::DEFAULT_IMAGE_HANDLE};
use bevy_ecs_ldtk::ldtk::{self, ldtk_fields::LdtkFields};
use bevy_rapier2d::prelude::*;

use super::LevelSize;
use crate::{
	game::grid_to_world,
	input::Action,
	player::{player_controls, Player},
	states::{AppState, Exit},
};

const SEGMENT_SIZE: f32 = 0.5;

pub struct RopePlugin;

impl Plugin for RopePlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_event::<SpawnRope>().add_systems(
			(
				spawn_rope,
				rope_spawn,
				update_rope,
				swing_controls.after(player_controls),
			)
				.distributive_run_if(in_state(AppState::Game)),
		);
	}
}

fn spawn_rope(
	mut ev_spawn_rope: EventWriter<SpawnRope>,
	level_size: LevelSize,
	q_spawned_ldtk_entities: Query<&ldtk::EntityInstance, Added<ldtk::EntityInstance>>,
) {
	for instance in q_spawned_ldtk_entities
		.iter()
		.filter(|e| e.identifier == "Rope")
	{
		if let Err(e) = (|| {
			let length = *instance.get_int_field("length")? as u32;
			let pos = grid_to_world(&level_size, instance.grid);

			ev_spawn_rope.send(SpawnRope { pos, length });
			Result::<_, Box<dyn Error>>::Ok(())
		})() {
			warn!("failed to spawn launchpad: {e}");
		}
	}
}

#[derive(Clone, Component)]
pub struct RopeAnchor;

#[derive(Clone, Component)]
pub struct RopeSegment;

pub struct SpawnRope {
	pos: Vec2,
	length: u32,
}

fn rope_spawn(mut commands: Commands, mut ev_spawn_rope: EventReader<SpawnRope>) {
	for spawn_rope in ev_spawn_rope.iter() {
		let mut anchor = commands
			.spawn((
				RopeAnchor,
				RigidBody::Fixed,
				SpatialBundle::from_transform(Transform::from_translation(
					spawn_rope.pos.extend(0.0),
				)),
				Exit(AppState::Game),
			))
			.id();
		for idx in 0..((spawn_rope.length as f32 / SEGMENT_SIZE) as usize) {
			let joint = if idx == 0 {
				RevoluteJointBuilder::new()
					.local_anchor1(Vec2::ZERO)
					.local_anchor2(Vec2::Y * SEGMENT_SIZE / 2.0)
			} else {
				RevoluteJointBuilder::new()
					.local_anchor1(Vec2::Y * -SEGMENT_SIZE / 2.0)
					.local_anchor2(Vec2::Y * SEGMENT_SIZE / 2.0)
			};
			anchor = commands
				.spawn((
					RopeSegment,
					RigidBody::Dynamic,
					ImpulseJoint::new(anchor, joint),
					Sensor,
					ColliderMassProperties::Mass(1.0 * SEGMENT_SIZE),
					Damping {
						linear_damping: 1.0,
						angular_damping: 2.0,
					},
					Collider::capsule(
						Vec2::Y * SEGMENT_SIZE / 2.0,
						-Vec2::Y * SEGMENT_SIZE / 2.0,
						1.0,
					),
					SpatialBundle::from_transform(Transform::from_translation(
						(spawn_rope.pos - (idx as f32 * Vec2::Y * SEGMENT_SIZE)).extend(0.0),
					)),
					Sprite {
						color: Color::rgb(0.25, 0.25, 0.75) * 3.0,
						custom_size: Some(Vec2::new(0.2, SEGMENT_SIZE)),
						..default()
					},
					DEFAULT_IMAGE_HANDLE.typed::<Image>(),
					CollidingEntities::default(),
					Exit(AppState::Game),
				))
				.id();
		}
	}
}

/// Apply forces on the player when they swing left & right
fn swing_controls(
	action: Res<Input<Action>>,
	mut q_player: Query<&mut ExternalForce, (With<Player>, With<ImpulseJoint>)>,
) {
	let Ok(mut ext_force) = q_player.get_single_mut() else {
		return;
	};

	ext_force.force = Vec2::ZERO;
	if action.pressed(Action::Left) {
		ext_force.force -= Vec2::X * 400.0;
	}
	if action.pressed(Action::Right) {
		ext_force.force += Vec2::X * 400.0;
	}
}

/// Attaches/Detaches the player when they press space
fn update_rope(
	mut commands: Commands,
	mut q_player: Query<(
		Entity,
		&mut Player,
		&mut CollidingEntities,
		Option<&ImpulseJoint>,
	)>,
	action: Res<Input<Action>>,
	q_rope: Query<(Entity, &Transform), (Without<Player>, With<RopeSegment>)>,
) {
	let Ok((player_entity, mut player, colliding_entities, maybe_joint)) = q_player.get_single_mut() else {
		return;
	};

	if action.just_pressed(Action::Jump) {
		if maybe_joint.is_some() {
			commands.entity(player_entity).remove::<ImpulseJoint>();
		} else if let Some((e, _)) = colliding_entities.iter().find_map(|e| q_rope.get(e).ok()) {
			let joint = RevoluteJointBuilder::new()
				.local_anchor1(Vec2::ZERO)
				.local_anchor2(Vec2::ZERO);
			commands
				.entity(player_entity)
				.insert(ImpulseJoint::new(e, joint));
			player.remaining_jumps = 1;
		}
	}
}
