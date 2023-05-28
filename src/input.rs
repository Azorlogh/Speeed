use std::collections::BTreeMap;

use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

pub const JOYSTICK_THRESHOLD: f32 = 0.8;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ButtonOrAxis {
	Button(GamepadButtonType),
	Axis(GamepadAxisType, bool), // false: positive, true: negative
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Mapping {
	pub key: KeyCode,
	pub button_or_axis: ButtonOrAxis,
}

#[derive(Clone, Serialize, Deserialize, Resource)]
pub struct InputMapping(pub BTreeMap<Action, Mapping>);

impl Default for InputMapping {
	fn default() -> Self {
		Self(BTreeMap::from([
			(
				Action::Jump,
				Mapping {
					key: KeyCode::Space,
					button_or_axis: ButtonOrAxis::Button(GamepadButtonType::South),
				},
			),
			(
				Action::Left,
				Mapping {
					key: KeyCode::Left,
					button_or_axis: ButtonOrAxis::Axis(GamepadAxisType::LeftStickX, true),
				},
			),
			(
				Action::Right,
				Mapping {
					key: KeyCode::Right,
					button_or_axis: ButtonOrAxis::Axis(GamepadAxisType::LeftStickX, false),
				},
			),
			(
				Action::GroundPound,
				Mapping {
					key: KeyCode::Down,
					button_or_axis: ButtonOrAxis::Button(GamepadButtonType::West),
				},
			),
			(
				Action::Restart,
				Mapping {
					key: KeyCode::R,
					button_or_axis: ButtonOrAxis::Button(GamepadButtonType::North),
				},
			),
			#[cfg(debug_assertions)]
			(
				Action::Skip,
				Mapping {
					key: KeyCode::N,
					button_or_axis: ButtonOrAxis::Button(GamepadButtonType::LeftTrigger),
				},
			),
		]))
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Resource)]
pub enum CurrentInputMode {
	Keyboard,
	Gamepad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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
	mapping: Res<InputMapping>,
	keys: Res<Input<KeyCode>>,
	mut input_mode: ResMut<CurrentInputMode>,
) {
	actions.clear();
	for (action, mapping) in &mapping.0 {
		if keys.just_pressed(mapping.key) {
			*input_mode = CurrentInputMode::Keyboard;
			actions.press(*action);
		}
		if keys.just_released(mapping.key) {
			actions.release(*action);
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct GamepadAxes(HashMap<GamepadAxisType, f32>);
pub fn handle_gamepad_input(
	mut actions: ResMut<Input<Action>>,
	mapping: Res<InputMapping>,
	gamepad_axes: Res<Axis<GamepadAxis>>,
	buttons: Res<Input<GamepadButton>>,
	gamepads: Res<Gamepads>,
	mut prev_axes: Local<GamepadAxes>,
	mut input_mode: ResMut<CurrentInputMode>,
) {
	let Some(gp) = gamepads.iter().next() else {
		return;
	};

	let mut axes = GamepadAxes::default();
	for axis in &[
		GamepadAxisType::LeftStickX,
		GamepadAxisType::LeftStickY,
		GamepadAxisType::LeftZ,
		GamepadAxisType::RightStickX,
		GamepadAxisType::RightStickY,
		GamepadAxisType::RightZ,
	] {
		axes.0.insert(
			*axis,
			gamepad_axes
				.get(GamepadAxis {
					gamepad: gp,
					axis_type: *axis,
				})
				.unwrap(),
		);
	}

	for (action, mapping) in &mapping.0 {
		match mapping.button_or_axis {
			ButtonOrAxis::Button(btn) => {
				if buttons.just_pressed(GamepadButton {
					gamepad: gp,
					button_type: btn,
				}) {
					*input_mode = CurrentInputMode::Gamepad;
					actions.press(*action);
				}
				if buttons.just_released(GamepadButton {
					gamepad: gp,
					button_type: btn,
				}) {
					actions.release(*action);
				}
			}
			ButtonOrAxis::Axis(axis, negative) => {
				if let Some(prev_value) = prev_axes.0.get(&axis) {
					if let Some(value) = axes.0.get(&axis) {
						if negative {
							if *prev_value >= -JOYSTICK_THRESHOLD && *value < -JOYSTICK_THRESHOLD {
								actions.press(*action);
							}
							if *prev_value < -JOYSTICK_THRESHOLD && *value >= -JOYSTICK_THRESHOLD {
								actions.release(*action);
							}
						} else {
							if *prev_value <= JOYSTICK_THRESHOLD && *value > JOYSTICK_THRESHOLD {
								actions.press(*action);
							}
							if *prev_value > JOYSTICK_THRESHOLD && *value <= JOYSTICK_THRESHOLD {
								actions.release(*action);
							}
						}
					}
				}
			}
		}
	}

	*prev_axes = axes.clone();
}
