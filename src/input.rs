use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(Input::<Action>::default())
			.add_system(handle_inputs);
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
	Jump,
	Left,
	Right,
	Restart,
}

#[derive(Clone, Debug, Default)]
pub struct GamepadAxes {
	horizontal: f32,
}
pub fn handle_inputs(
	mut actions: ResMut<Input<Action>>,
	gamepad_axes: Res<Axis<GamepadAxis>>,
	buttons: Res<Input<GamepadButton>>,
	gamepads: Res<Gamepads>,
	keys: Res<Input<KeyCode>>,
	mut prev_axes: Local<GamepadAxes>,
) {
	let gamepad = gamepads.iter().next();
	actions.clear();
	let mut axes = GamepadAxes::default();
	if let Some(gp) = gamepad {
		axes.horizontal = gamepad_axes
			.get(GamepadAxis {
				gamepad: gp,
				axis_type: GamepadAxisType::LeftStickX,
			})
			.unwrap();
	}
	if keys.just_pressed(KeyCode::Space)
		|| gamepad
			.map(|gp| {
				buttons.just_pressed(GamepadButton {
					gamepad: gp,
					button_type: GamepadButtonType::South,
				})
			})
			.unwrap_or(false)
	{
		actions.press(Action::Jump);
	}
	if keys.just_released(KeyCode::Space)
		|| gamepad
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

	if keys.just_pressed(KeyCode::R)
		|| gamepad
			.map(|gp| {
				buttons.just_pressed(GamepadButton {
					gamepad: gp,
					button_type: GamepadButtonType::East,
				})
			})
			.unwrap_or(false)
	{
		actions.press(Action::Restart);
	}
	if keys.just_released(KeyCode::R)
		|| gamepad
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

	const STICK_THESHOLD: f32 = 0.8;
	if keys.just_pressed(KeyCode::Left)
		|| (prev_axes.horizontal >= -STICK_THESHOLD && axes.horizontal < -STICK_THESHOLD)
	{
		actions.press(Action::Left);
	}
	if keys.just_released(KeyCode::Left)
		|| (prev_axes.horizontal < -STICK_THESHOLD && axes.horizontal >= -STICK_THESHOLD)
	{
		actions.release(Action::Left);
	}
	if keys.just_pressed(KeyCode::Right)
		|| (prev_axes.horizontal <= STICK_THESHOLD && axes.horizontal > STICK_THESHOLD)
	{
		actions.press(Action::Right);
	}
	if keys.just_released(KeyCode::Right)
		|| (prev_axes.horizontal > STICK_THESHOLD && axes.horizontal <= STICK_THESHOLD)
	{
		actions.release(Action::Right);
	}
	*prev_axes = axes.clone();
}
