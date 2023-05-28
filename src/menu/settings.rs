use bevy::ecs::prelude::*;
use bevy_iced::{
	iced::{
		widget::{text, Button, Column, Row},
		Alignment, Length,
	},
	IcedContext,
};
use bevy_input::prelude::*;

use super::MenuState;
use crate::{
	input::{Action, ButtonOrAxis, InputMapping, JOYSTICK_THRESHOLD},
	settings::SaveSettings,
};

#[derive(Clone)]
pub enum SettingsUiMessage {
	EditMapping(Action, InputMode),
	ResetMappings,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum InputMode {
	Keyboard,
	Gamepad,
}

#[derive(Default)]
pub struct SettingsMenuState {
	editing_action: Option<(Action, InputMode)>,
}

pub fn settings_update(
	mut messages: EventReader<SettingsUiMessage>,
	mut menu_state: ResMut<MenuState>,
	mut mappings: ResMut<InputMapping>,
) {
	let MenuState::Settings(state) = menu_state.as_mut() else {
		return;
	};

	for msg in messages.iter() {
		match msg {
			SettingsUiMessage::EditMapping(action, input_mode) => {
				state.editing_action = Some((*action, *input_mode));
			}
			SettingsUiMessage::ResetMappings => {
				*mappings = InputMapping::default();
			}
		}
	}
}

pub fn settings_ui(
	mut ctx: IcedContext<SettingsUiMessage>,
	mut menu_state: ResMut<MenuState>,
	mut mappings: ResMut<InputMapping>,
	keys: Res<Input<KeyCode>>,
	gamepad_axes: Res<Axis<GamepadAxis>>,
	buttons: Res<Input<GamepadButton>>,
	mut ev_save_settings: EventWriter<SaveSettings>,
) {
	let MenuState::Settings(state) = menu_state.as_mut() else {
		return;
	};

	let mut mappings_col = Column::new().align_items(Alignment::Center);
	for (action, mapping) in &mappings.0 {
		let mut row = Row::new().push(text(format!("{:?}", action)).width(128.0));
		row = row.push(
			if state.editing_action == Some((*action, InputMode::Keyboard)) {
				Button::new("?")
			} else {
				Button::new(text(format!("{:?}", mapping.key)))
					.on_press(SettingsUiMessage::EditMapping(*action, InputMode::Keyboard))
			}
			.width(128.0),
		);
		row = row.push(
			if state.editing_action == Some((*action, InputMode::Gamepad)) {
				Button::new("?")
			} else {
				Button::new(text(format!("{:?}", mapping.button_or_axis)))
					.on_press(SettingsUiMessage::EditMapping(*action, InputMode::Gamepad))
			}
			.width(192.0),
		);
		mappings_col = mappings_col.push(row);
	}

	ctx.display(
		Column::new()
			.align_items(Alignment::Center)
			.width(Length::Fill)
			.padding(64.0)
			.spacing(32.0)
			.push(text("Settings").size(30.0))
			.push(mappings_col)
			.push(Button::new("Reset to defaults").on_press(SettingsUiMessage::ResetMappings)),
	);

	if let Some((action, input_mode)) = state.editing_action.clone() {
		match input_mode {
			InputMode::Keyboard => {
				if let Some(key) = keys.get_just_pressed().next() {
					let mapping = mappings.0.get_mut(&action).unwrap();
					mapping.key = *key;
					state.editing_action = None;
				}
			}
			InputMode::Gamepad => {
				if let Some(btn) = buttons.get_just_pressed().next() {
					let mapping = mappings.0.get_mut(&action).unwrap();
					mapping.button_or_axis = ButtonOrAxis::Button(btn.button_type);
					state.editing_action = None;
				}
				for axis in gamepad_axes.devices() {
					let value = gamepad_axes.get(*axis).unwrap();
					if value > JOYSTICK_THRESHOLD {
						let mapping = mappings.0.get_mut(&action).unwrap();
						mapping.button_or_axis = ButtonOrAxis::Axis(axis.axis_type, false);
						state.editing_action = None;
					}
					if value < -JOYSTICK_THRESHOLD {
						let mapping = mappings.0.get_mut(&action).unwrap();
						mapping.button_or_axis = ButtonOrAxis::Axis(axis.axis_type, true);
						state.editing_action = None;
					}
				}
			}
		}
	} else {
		if keys.just_pressed(KeyCode::Escape) {
			ev_save_settings.send(SaveSettings);
			*menu_state = MenuState::Main;
			return;
		}
	}
}
