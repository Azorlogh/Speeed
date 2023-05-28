use std::{fs::read_to_string, path::PathBuf};

use anyhow::Result;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::input::InputMapping;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
	fn build(&self, app: &mut App) {
		let settings = load_settings();
		app.insert_resource(settings.input_mapping)
			.add_event::<SaveSettings>()
			.add_system(settings_save);
	}
}

fn settings_path() -> PathBuf {
	directories::ProjectDirs::from("", "Azorlogh", "Speeed")
		.unwrap()
		.config_dir()
		.join("settings.toml")
}

#[derive(Default, Serialize, Deserialize)]
pub struct Settings {
	input_mapping: InputMapping,
}

fn load_settings() -> Settings {
	match try_load_settings() {
		Ok(s) => s,
		Err(e) => {
			warn!("failed to load settings, using defaults: {e}");
			default()
		}
	}
}

fn try_load_settings() -> Result<Settings> {
	let s = read_to_string(settings_path())?;
	Ok(serde_json::from_str(&s)?)
}

pub struct SaveSettings;

fn settings_save(
	mut ev_save_settings: EventReader<SaveSettings>,
	input_mapping: Res<InputMapping>,
) {
	if ev_save_settings.iter().count() > 0 {
		if let Err(e) = try_save(Settings {
			input_mapping: input_mapping.clone(),
		}) {
			error!("failed to save settings: {e:?}")
		}
	}
}

fn try_save(settings: Settings) -> Result<()> {
	let s = serde_json::to_string_pretty(&settings)?;
	let path = settings_path();
	std::fs::create_dir_all(path.parent().unwrap())?;
	std::fs::write(path, s)?;
	Ok(())
}
