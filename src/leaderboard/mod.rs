use std::{error::Error, time::Duration};

use bevy::prelude::*;
use bevy_ecs_ldtk::{LevelSelection, LevelSet, Respawn};
use bevy_egui::{
	egui::{self, Align, Color32, Layout},
	EguiContexts,
};
use serde::{Deserialize, Serialize};

use crate::{input::Action, states::AppState};

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(Leaderboard::load())
			.add_system(setup.in_schedule(OnEnter(AppState::Leaderboard)))
			.add_system(exit.in_schedule(OnExit(AppState::Leaderboard)))
			.add_system(leaderboard_ui.run_if(in_state(AppState::Leaderboard)));
	}
}

/// A score is counted as a number of milliseconds
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Resource, Serialize, Deserialize)]
pub struct Score(pub u64);

impl std::fmt::Display for Score {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			humantime::format_duration(Duration::from_millis(self.0))
		)
	}
}

#[derive(Resource, Serialize, Deserialize)]
pub struct Leaderboard(Vec<Vec<Score>>);

impl Leaderboard {
	pub fn load() -> Self {
		Self::try_load().unwrap_or(Leaderboard(vec![vec![]; 6]))
	}

	pub fn try_load() -> Result<Self, Box<dyn Error>> {
		let s = std::fs::read_to_string("leaderboard.json")?;
		Ok(serde_json::from_str(&s)?)
	}

	pub fn save(&self) -> Result<(), Box<dyn Error>> {
		let s = serde_json::to_string(self)?;
		std::fs::write("leaderboard.json", s)?;
		Ok(())
	}

	pub fn add_score(&mut self, level: usize, score: Score) {
		let pos = self.0[level].binary_search(&score).unwrap_or_else(|e| e);
		self.0[level].insert(pos, score);
		self.0[level].truncate(10);
		if let Err(e) = self.save() {
			warn!("failed to save leadeboard: {e}");
		}
	}
}

#[derive(Resource)]
pub struct CurrentScore(pub Score);

#[derive(Component)]
pub struct RestartButton;
#[derive(Component)]
pub struct NextButton;

fn setup(mut commands: Commands) {
	let mut camera = Camera2dBundle::default();
	camera.transform.translation.z = -10000.0;
	commands.spawn(camera);
}

fn leaderboard_ui(
	mut commands: Commands,
	score: Res<CurrentScore>,
	leaderboard: Res<Leaderboard>,
	mut egui_ctx: EguiContexts,
	mut next_app_state: ResMut<NextState<AppState>>,
	mut level_selection: ResMut<LevelSelection>,
	q_ldtk_world: Query<Entity, With<LevelSet>>,
	actions: Res<Input<Action>>,
) {
	let LevelSelection::Index(level) = level_selection.clone() else {
		panic!("expected level index");
	};

	let idx = leaderboard.0[level]
		.iter()
		.enumerate()
		.filter(|(_, v)| score.0 == **v)
		.map(|(i, _)| i)
		.last();

	egui::CentralPanel::default().show(egui_ctx.ctx_mut(), |ui| {
		ui.vertical_centered(|ui| {
			let msg = match idx.is_some() {
				true => "Well played!",
				false => "Better luck next time :)",
			};
			ui.label(msg);
			ui.heading(&score.0.to_string());
			ui.group(|ui| {
				for (i, s) in leaderboard.0[level].iter().enumerate() {
					let style = ui.style_mut();

					if Some(i) == idx {
						style.visuals.override_text_color = Some(Color32::RED);
					} else {
						style.visuals.override_text_color = None;
					};
					ui.label(&s.to_string());
				}
			});
			ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
				if ui.button("Next").clicked() || actions.just_pressed(Action::Jump) {
					*level_selection = LevelSelection::Index((level + 1) % 7);
					next_app_state.set(AppState::Game);
				}
				if ui.button("Restart").clicked() || actions.just_pressed(Action::GroundPound) {
					let world = q_ldtk_world.single();
					commands.entity(world).insert(Respawn);
					next_app_state.set(AppState::Game);
				}
			});
		});
	});
}

fn exit(mut commands: Commands, q_nodes: Query<Entity, Or<(With<Node>, With<Camera>)>>) {
	for node in q_nodes.iter() {
		commands.entity(node).despawn();
	}
}
