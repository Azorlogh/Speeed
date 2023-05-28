use bevy::prelude::*;
use bevy_egui::{
	egui::{self, Align},
	EguiContexts,
};

use super::MenuState;
use crate::{
	input::{Action, ButtonOrAxis, InputMapping, JOYSTICK_THRESHOLD},
	settings::SaveSettings,
};

#[derive(PartialEq, Eq, Clone)]
pub enum InputMode {
	Keyboard,
	Gamepad,
}

#[derive(Default)]
pub struct SettingsMenuState {
	editing_action: Option<(Action, InputMode)>,
}

pub fn settings_ui(
	mut egui_ctx: EguiContexts,
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

	egui::CentralPanel::default().show(egui_ctx.ctx_mut(), |ui| {
		ui.label("Settings");
		ui.add_space(32.0);
		for (action, mapping) in &mappings.0 {
			ui.with_layout(egui::Layout::left_to_right(Align::LEFT), |ui| {
				ui.label(format!("{action:?}"));
				if state.editing_action == Some((*action, InputMode::Keyboard)) {
					ui.button("?").clicked(); // do nothing
				} else {
					if ui.button(format!("{:?}", mapping.key)).clicked() {
						state.editing_action = Some((*action, InputMode::Keyboard));
					}
				}
				if state.editing_action == Some((*action, InputMode::Gamepad)) {
					ui.button("?").clicked(); // do nothing
				} else {
					if ui.button(format!("{:?}", mapping.button_or_axis)).clicked() {
						state.editing_action = Some((*action, InputMode::Gamepad));
					}
				}
			});
		}
		if ui.button("Reset to defaults").clicked() {
			*mappings = InputMapping::default();
		}
	});

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
