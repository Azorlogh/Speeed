use std::error::Error;

use bevy::prelude::*;
use bevy_ecs_ldtk::{
	ldtk::{self, ldtk_fields::LdtkFields},
	LayerMetadata,
};
use bevy_hanabi::{
	AccelModifier, ColorOverLifetimeModifier, EffectAsset, Gradient, InitLifetimeModifier,
	InitPositionCircleModifier, ParticleEffect, ShapeDimension, SizeOverLifetimeModifier, Spawner,
};
use bevy_rapier2d::prelude::*;

use crate::{
	game::grid_to_world,
	player::Player,
	states::{AppState, Exit},
};

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_event::<SpawnPortal>().add_systems(
			(spawn_portal, portal_spawn, update_portal)
				.distributive_run_if(in_state(AppState::Game)),
		);
	}
}

fn spawn_portal(
	mut commands: Commands,
	mut effects: ResMut<Assets<EffectAsset>>,
	mut ev_spawn_portal: EventWriter<SpawnPortal>,
	q_layer: Query<&LayerMetadata>,
	q_spawned_ldtk_entities: Query<&ldtk::EntityInstance, Added<ldtk::EntityInstance>>,
) {
	for instance in q_spawned_ldtk_entities
		.iter()
		.filter(|e| e.identifier == "Portal")
	{
		if let Err(e) = (|| {
			let dest = grid_to_world(q_layer.single(), *instance.get_point_field("destination")?);
			let angle_in = instance.get_float_field("angle_in")?.to_radians();
			let angle_out = instance.get_float_field("angle_out")?.to_radians();
			let pos = grid_to_world(q_layer.single(), instance.grid);
			let delta = dest - pos;

			ev_spawn_portal.send(SpawnPortal {
				pos,
				portal: Portal {
					delta,
					angle_in,
					angle_out,
				},
			});
			Result::<_, Box<dyn Error>>::Ok(())
		})() {
			warn!("failed to spawn launchpad: {e}");
		}
	}
}

const LAUNCHPAD_SIZE: f32 = 1.0;

#[derive(Clone, Component)]
pub struct Portal {
	delta: Vec2,
	angle_in: f32,
	angle_out: f32,
}

pub struct SpawnPortal {
	pos: Vec2,
	portal: Portal,
}

fn portal_spawn(
	mut commands: Commands,
	mut effects: ResMut<Assets<EffectAsset>>,
	mut ev_spawn_portal: EventReader<SpawnPortal>,
) {
	for spawn_portal in ev_spawn_portal.iter() {
		let mut gradient = Gradient::new();
		gradient.add_key(0.0, Vec4::new(0.1, 0.3, 0.9, 1.0));
		gradient.add_key(1.0, Vec4::new(0.1, 0.3, 0.9, 0.0));
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
				axis: Vec2::from_angle(spawn_portal.portal.angle_in).extend(0.0),
				radius: 1.5,
				dimension: ShapeDimension::Surface,
			})
			.init(InitLifetimeModifier {
				lifetime: 0.5f32.into(),
			})
			.update(AccelModifier::constant(
				(Vec2::from_angle(spawn_portal.portal.angle_in) * 10.0)
					.extend(0.0)
					.into(),
			))
			.render(SizeOverLifetimeModifier {
				gradient: Gradient::constant(Vec2::splat(0.5)),
			})
			.render(ColorOverLifetimeModifier { gradient }),
		);
		commands.spawn((
			ParticleEffect::new(effect).with_z_layer_2d(Some(0.1)),
			spawn_portal.portal.clone(),
			SpatialBundle::from_transform(Transform::from_translation(
				spawn_portal.pos.extend(0.0),
			)),
			Collider::segment(Vec2::X * -1.5, Vec2::X * 1.5),
			Exit(AppState::Game),
		));

		let mut gradient = Gradient::new();
		gradient.add_key(0.0, Vec4::new(0.9, 0.7, 0.2, 1.0));
		gradient.add_key(1.0, Vec4::new(0.9, 0.7, 0.2, 0.0));
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
				axis: Vec2::from_angle(spawn_portal.portal.angle_out).extend(0.0),
				radius: 1.5,
				dimension: ShapeDimension::Surface,
			})
			.init(InitLifetimeModifier {
				lifetime: 0.5f32.into(),
			})
			.update(AccelModifier::constant(
				(Vec2::from_angle(spawn_portal.portal.angle_out) * 10.0)
					.extend(0.0)
					.into(),
			))
			.render(SizeOverLifetimeModifier {
				gradient: Gradient::constant(Vec2::splat(0.5)),
			})
			.render(ColorOverLifetimeModifier { gradient }),
		);
		// Out portal
		commands.spawn((
			ParticleEffect::new(effect).with_z_layer_2d(Some(0.1)),
			SpatialBundle::from_transform(Transform::from_translation(
				(spawn_portal.pos + spawn_portal.portal.delta).extend(0.0),
			)),
			Exit(AppState::Game),
		));
	}
}

fn update_portal(
	mut ev_collision: EventReader<CollisionEvent>,
	mut q_player: Query<(Entity, &mut Transform, &mut Velocity), With<Player>>,
	q_portal: Query<(Entity, &Transform, &Portal), Without<Player>>,
) {
	let Ok((player_entity, mut player_tr, mut player_vel)) = q_player.get_single_mut() else {
		return;
	};
	// let Ok((portal_entity, portal_tr, portal)) = q_portal.get_single() else {
	// 	return;
	// };

	for collision_event in ev_collision.iter() {
		match collision_event {
			CollisionEvent::Started(e0, e1, _) => {
				if [*e0, *e1].contains(&player_entity) {
					if let Some((_, portal_tr, portal)) =
						[e0, e1].iter().find_map(|e| q_portal.get(**e).ok())
					{
						let offset =
							player_tr.translation.truncate() - portal_tr.translation.truncate();

						let angle = Vec2::from_angle(portal.angle_in - portal.angle_out);

						player_tr.translation =
							portal_tr.translation
								+ portal.delta.extend(0.0) + angle.rotate(offset).extend(0.0);
						player_vel.linvel = angle.rotate(player_vel.linvel);
					}
				}
			}
			_ => {}
		}
	}
}

// fn update_portal(
// 	mut q_player: Query<(&mut Transform, &mut Velocity), With<Player>>,
// 	q_portal: Query<(&Transform, &Portal), Without<Player>>,
// ) {
// 	let Ok((mut player_tr, mut player_vel)) = q_player.get_single_mut() else {
// 		return;
// 	};

// 	for (launchpad_tr, portal) in &q_portal {
// 		if player_tr
// 			.translation
// 			.truncate()
// 			.distance(launchpad_tr.translation.truncate())
// 			<= LAUNCHPAD_SIZE
// 		{
// 			player_tr.translation += portal.delta.extend(0.0);
// 			player_vel.linvel =
// 				Vec2::from_angle(portal.angle_in - portal.angle_out).rotate(player_vel.linvel);
// 		}
// 	}
// }
