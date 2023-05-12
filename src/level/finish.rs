use bevy::{prelude::*, render::texture::DEFAULT_IMAGE_HANDLE, sprite::Sprite};
use bevy_rapier2d::prelude::Collider;

const FINISH_SIZE: f32 = 2.0;

#[derive(Component)]
pub struct Finish;

#[derive(Bundle)]
pub struct FinishBundle {
	pub finish: Finish,
	pub sprite: Sprite,
	pub spatial: SpatialBundle,
	pub texture: Handle<Image>,
	pub collider: Collider,
}
impl Default for FinishBundle {
	fn default() -> Self {
		Self {
			finish: Finish,
			sprite: Sprite {
				color: Color::rgb(1.0, 0.25, 0.5),
				custom_size: Some(Vec2::splat(FINISH_SIZE)),
				..default()
			},
			spatial: SpatialBundle::default(),
			texture: DEFAULT_IMAGE_HANDLE.typed(),
			collider: Collider::ball(FINISH_SIZE / 2.0),
		}
	}
}
