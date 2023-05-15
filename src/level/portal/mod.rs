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
		app.add_systems(
			(spawn_portal, update_portal).distributive_run_if(in_state(AppState::Game)),
		);
	}
}

fn spawn_portal(
	mut commands: Commands,
	mut effects: ResMut<Assets<EffectAsset>>,
	q_layer: Query<&LayerMetadata>,
	q_spawned_ldtk_entities: Query<&ldtk::EntityInstance, Added<ldtk::EntityInstance>>,
) {
	for instance in q_spawned_ldtk_entities
		.iter()
		.filter(|e| e.identifier == "Portal")
	{
		if let Err(e) = (|| {
			let dest = *instance.get_point_field("x")?;
			commands.spawn((
				PortalBundle::new(
					grid_to_world(q_layer.single(), instance.grid),
					Vec2::new(x, y),
					&mut effects,
				),
				Exit(AppState::Game),
			));
			Result::<_, Box<dyn Error>>::Ok(())
		})() {
			warn!("failed to spawn launchpad: {e}");
		}
	}
}

const LAUNCHPAD_SIZE: f32 = 1.0;

#[derive(Component)]
pub struct Portal {
	vel: Vec2,
}

#[derive(Bundle)]
pub struct PortalBundle {
	pub finish: Portal,
	pub spatial: SpatialBundle,
	// pub collider: Collider,
	pub effect: ParticleEffect,
}
impl PortalBundle {
	fn new(pos: Vec2, vel: Vec2, effects: &mut Assets<EffectAsset>) -> Self {
		let mut gradient = Gradient::new();
		gradient.add_key(0.0, Vec4::new(0.1, 0.3, 0.9, 1.0));
		gradient.add_key(1.0, Vec4::new(0.1, 0.3, 0.9, 0.0));

		let spawner = Spawner::rate(10.0.into());
		let effect = effects.add(
			EffectAsset {
				name: "FinishEffect".into(),
				capacity: 4096,
				spawner,
				..Default::default()
			}
			.init(InitPositionCircleModifier {
				center: Vec3::ZERO,
				axis: Vec3::Y,
				radius: 0.2,
				dimension: ShapeDimension::Surface,
			})
			.init(InitLifetimeModifier {
				lifetime: 0.5f32.into(),
			})
			.update(AccelModifier::constant(vel.extend(0.0).into()))
			.render(SizeOverLifetimeModifier {
				gradient: Gradient::constant(Vec2::splat(0.5)),
			})
			.render(ColorOverLifetimeModifier { gradient }),
		);

		Self {
			effect: ParticleEffect::new(effect).with_z_layer_2d(Some(0.1)),
			finish: Portal { vel },
			spatial: SpatialBundle::from_transform(Transform::from_translation(pos.extend(0.0))),
		}
	}
}

fn update_portal(
	mut q_player: Query<&mut Transform, With<Player>>,
	q_portal: Query<(&Transform, &Portal), Without<Player>>,
) {
	let Ok(player_tr) = q_player.get_single_mut() else {
		return;
	};

	for (launchpad_tr, launchpad) in &q_portal {
		if player_tr
			.translation
			.truncate()
			.distance(launchpad_tr.translation.truncate())
			<= LAUNCHPAD_SIZE
		{
			player_vel.linvel = launchpad.vel;
		}
	}
}