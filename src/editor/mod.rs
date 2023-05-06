use bevy::prelude::*;

use crate::states::AppState;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
	fn build(&self, app: &mut App) {
		app.add_system(setup.in_schedule(OnEnter(AppState::Editor)));
	}
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2dBundle::default());
}
