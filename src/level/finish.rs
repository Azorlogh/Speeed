use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_rapier2d::prelude::Collider;

use crate::{
	game::grid_to_world,
	states::{AppState, Exit},
};

const FINISH_SIZE: f32 = 2.0;

#[derive(Component)]
pub struct Finish;

#[derive(Bundle)]
pub struct FinishBundle {
	pub finish: Finish,
	pub spatial: SpatialBundle,
	pub collider: Collider,
}
impl Default for FinishBundle {
	fn default() -> Self {
		Self {
			finish: Finish,
			spatial: SpatialBundle::default(),
			collider: Collider::ball(FINISH_SIZE / 2.0),
		}
	}
}

pub fn spawn_finish(
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
				axis: Vec3::Z,
				radius: 0.5,
				dimension: ShapeDimension::Surface,
			})
			.init(InitVelocityCircleModifier {
				center: Vec3::ZERO,
				axis: Vec3::Z,
				speed: (-0.2f32).into(),
			})
			.init(InitLifetimeModifier {
				lifetime: 5_f32.into(),
			})
			.render(SizeOverLifetimeModifier {
				gradient: Gradient::constant(Vec2::splat(0.5)),
			})
			.render(ColorOverLifetimeModifier { gradient }),
		);

		commands
			.spawn(ParticleEffectBundle {
				effect: ParticleEffect::new(effect).with_z_layer_2d(Some(0.1)),
				..default()
			})
			.insert((
				FinishBundle {
					spatial: SpatialBundle::from_transform(Transform::from_translation(
						grid_to_world(q_layer.single(), finish.grid).extend(0.0),
					)),
					..default()
				},
				Exit(AppState::Game),
			));
	}
}
