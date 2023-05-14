use bevy::prelude::*;
use bevy_rapier2d::prelude::Collider;

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
