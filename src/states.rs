use bevy::prelude::States;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, States)]
pub enum AppState {
	#[default]
	Menu,
	Game,
	Editor,
}
