use bevy::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default, States)]
pub enum AppState {
	Menu,
	#[default]
	Game,
	Leaderboard,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
	fn build(&self, app: &mut App) {
		app.add_state::<AppState>()
			.add_system(exit_state.in_base_set(CoreSet::PostUpdate));
	}
}

#[derive(Component)]
pub struct Exit(pub AppState);

/// Automatically removes entities marked with the Exit component when changing state
pub fn exit_state(
	mut commands: Commands,
	state: Res<State<AppState>>,
	next_state: Res<NextState<AppState>>,
	q_entities: Query<(Entity, &Exit)>,
) {
	if next_state.0.is_some() {
		for (entity, _) in q_entities
			.iter()
			.filter(|(_, exit_state)| exit_state.0 == state.0)
		{
			commands.entity(entity).despawn_recursive();
		}
	}
}
