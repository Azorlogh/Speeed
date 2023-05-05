use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{player, states::AppState};

pub struct GamePlugin;

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app.add_system(setup.in_schedule(OnEnter(AppState::Game)));
	}
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2dBundle::default());

	// Player
	commands.spawn(player::PlayerBundle::default());

	// Floor
	commands.spawn((
		SpriteBundle {
			sprite: Sprite {
				color: Color::rgb(0.25, 0.25, 0.75),
				custom_size: Some(Vec2::new(400.0, 50.0)),
				..default()
			},
			transform: Transform::from_translation(Vec3::new(0.0, -200.0, 0.)),
			..default()
		},
		RigidBody::Fixed,
		Collider::cuboid(200.0, 25.0),
		Friction {
			coefficient: 0.0,
			combine_rule: default(),
		},
	));
}
