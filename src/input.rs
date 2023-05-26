use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, SystemSet)]
pub struct InputSet;

pub struct InputPlugin;

impl Plugin for InputPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(Input::<Action>::default())
			.insert_resource(CurrentInputMode::Keyboard)
			.add_systems(
				(handle_keyboard_input, handle_gamepad_input)
					.chain()
					.in_set(InputSet),
			);
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Resource)]
pub enum CurrentInputMode {
	Keyboard,
	Gamepad,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
	Jump,
	Left,
	Right,
	GroundPound,
	Restart,
	#[cfg(debug_assertions)]
	Skip,
}

pub fn handle_keyboard_input(
	mut actions: ResMut<Input<Action>>,
	keys: Res<Input<KeyCode>>,
	mut input_mode: ResMut<CurrentInputMode>,
) {
	actions.clear();
	let mut press = |actions: &mut Input<Action>, action| {
		*input_mode = CurrentInputMode::Keyboard;
		actions.press(action);
	};
	if keys.just_pressed(KeyCode::Space) {
		press(&mut actions, Action::Jump);
	}
	if keys.just_released(KeyCode::Space) {
		actions.release(Action::Jump);
	}

	if keys.just_pressed(KeyCode::R) {
		press(&mut actions, Action::Restart);
	}
	if keys.just_released(KeyCode::R) {
		actions.release(Action::Restart);
	}

	if keys.just_pressed(KeyCode::Down) {
		press(&mut actions, Action::GroundPound);
	}
	if keys.just_released(KeyCode::Down) {
		actions.release(Action::GroundPound);
	}

	if keys.just_pressed(KeyCode::Left) {
		press(&mut actions, Action::Left);
	}
	if keys.just_released(KeyCode::Left) {
		actions.release(Action::Left);
	}
	if keys.just_pressed(KeyCode::Right) {
		press(&mut actions, Action::Right);
	}
	if keys.just_released(KeyCode::Right) {
		actions.release(Action::Right);
	}
	#[cfg(debug_assertions)]
	{
		if keys.just_pressed(KeyCode::N) {
			actions.press(Action::Skip);
		}
		if keys.just_released(KeyCode::N) {
			actions.release(Action::Skip);
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct GamepadAxes {
	horizontal: f32,
}
pub fn handle_gamepad_input(
	mut actions: ResMut<Input<Action>>,
	gamepad_axes: Res<Axis<GamepadAxis>>,
	buttons: Res<Input<GamepadButton>>,
	gamepads: Res<Gamepads>,
	mut prev_axes: Local<GamepadAxes>,
	mut input_mode: ResMut<CurrentInputMode>,
) {
	let mut press = |actions: &mut Input<Action>, action| {
		*input_mode = CurrentInputMode::Gamepad;
		actions.press(action);
	};

	let gamepad = gamepads.iter().next();
	let mut axes = GamepadAxes::default();
	if let Some(gp) = gamepad {
		axes.horizontal = gamepad_axes
			.get(GamepadAxis {
				gamepad: gp,
				axis_type: GamepadAxisType::LeftStickX,
			})
			.unwrap();
	}
	if gamepad
		.map(|gp| {
			buttons.just_pressed(GamepadButton {
				gamepad: gp,
				button_type: GamepadButtonType::South,
			})
		})
		.unwrap_or(false)
	{
		press(&mut actions, Action::Jump);
	}
	if gamepad
		.map(|gp| {
			buttons.just_released(GamepadButton {
				gamepad: gp,
				button_type: GamepadButtonType::South,
			})
		})
		.unwrap_or(false)
	{
		actions.release(Action::Jump);
	}

	if gamepad
		.map(|gp| {
			buttons.just_pressed(GamepadButton {
				gamepad: gp,
				button_type: GamepadButtonType::East,
			})
		})
		.unwrap_or(false)
	{
		press(&mut actions, Action::Restart);
	}
	if gamepad
		.map(|gp| {
			buttons.just_released(GamepadButton {
				gamepad: gp,
				button_type: GamepadButtonType::East,
			})
		})
		.unwrap_or(false)
	{
		actions.release(Action::Restart);
	}

	if gamepad
		.map(|gp| {
			buttons.just_pressed(GamepadButton {
				gamepad: gp,
				button_type: GamepadButtonType::West,
			})
		})
		.unwrap_or(false)
	{
		press(&mut actions, Action::GroundPound);
	}
	if gamepad
		.map(|gp| {
			buttons.just_released(GamepadButton {
				gamepad: gp,
				button_type: GamepadButtonType::West,
			})
		})
		.unwrap_or(false)
	{
		actions.release(Action::GroundPound);
	}

	const STICK_THESHOLD: f32 = 0.8;
	if prev_axes.horizontal >= -STICK_THESHOLD && axes.horizontal < -STICK_THESHOLD {
		press(&mut actions, Action::Left);
	}
	if prev_axes.horizontal < -STICK_THESHOLD && axes.horizontal >= -STICK_THESHOLD {
		actions.release(Action::Left);
	}
	if prev_axes.horizontal <= STICK_THESHOLD && axes.horizontal > STICK_THESHOLD {
		press(&mut actions, Action::Right);
	}
	if prev_axes.horizontal > STICK_THESHOLD && axes.horizontal <= STICK_THESHOLD {
		actions.release(Action::Right);
	}

	*prev_axes = axes.clone();
}
