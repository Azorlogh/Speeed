///
/// The leaderboard menu at the end of a run.
///
use std::error::Error;

use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_ldtk::{LdtkAsset, LevelSelection, LevelSet, Respawn};
use bevy_egui::{
	egui::{self, Align, Color32, Layout},
	EguiContexts,
};
use serde::{Deserialize, Serialize};

use crate::{
	input::Action,
	states::{AppState, Exit},
};

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(Leaderboard::load())
			.insert_resource(Nickname(String::from("Default Nickname")))
			.add_system(setup.in_schedule(OnEnter(AppState::Leaderboard)))
			.add_system(exit.in_schedule(OnExit(AppState::Leaderboard)))
			.add_system(leaderboard_ui.run_if(in_state(AppState::Leaderboard)));
	}
}

#[derive(Resource)]
pub struct Nickname(pub String);

/// A score is counted as a number of milliseconds
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Resource, Serialize, Deserialize)]
pub struct Score(pub u64);

impl std::fmt::Display for Score {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let millis = self.0 % 1000;
		let seconds = (self.0 / 1000) % 60;
		let minutes = self.0 / 1000 / 60;
		write!(f, "{minutes:02}:{seconds:02}.{millis:03}")
	}
}

/// Leaderboard resource that also handles saving/loading to disk
#[derive(Resource, Serialize, Deserialize)]
pub struct Leaderboard(
	pub Vec<HashMap<String, Score>>, /* For each level, store each player's best score */
);

impl Leaderboard {
	pub fn load() -> Self {
		Self::try_load().unwrap_or(Leaderboard(vec![default(); 100]))
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

	pub fn add_score(&mut self, level: usize, nickname: &str, score: Score) {
		let best = self.0[level]
			.entry(nickname.to_owned())
			.or_insert(Score(u64::MAX));
		*best = (*best).min(score);
		if let Err(e) = self.save() {
			warn!("failed to save leadeboard: {e}");
		}
	}

	pub fn get_scores(&self, level: usize) -> Vec<(String, Score)> {
		let mut scores: Vec<(String, Score)> =
			self.0[level].iter().map(|(n, s)| (n.clone(), *s)).collect();
		scores.sort_by_key(|a| a.1);
		scores
	}
}

/// Score for the current finished run
#[derive(Resource)]
pub struct CurrentScore(pub Score);

fn setup(mut commands: Commands) {
	let mut camera = Camera2dBundle::default();
	camera.transform.translation.z = -10000.0;
	commands.spawn((camera, Exit(AppState::Leaderboard)));
}

fn leaderboard_ui(
	mut commands: Commands,
	current_score: Res<CurrentScore>,
	leaderboard: Res<Leaderboard>,
	mut egui_ctx: EguiContexts,
	mut next_app_state: ResMut<NextState<AppState>>,
	mut level_selection: ResMut<LevelSelection>,
	q_ldtk_world: Query<(Entity, &Handle<LdtkAsset>), With<LevelSet>>,
	actions: Res<Input<Action>>,
	ldtk_asset: Res<Assets<LdtkAsset>>,
	nickname: Res<Nickname>,
) {
	let LevelSelection::Index(level) = level_selection.clone() else {
		panic!("expected level index");
	};

	let improved = leaderboard.0[level].get(&nickname.0) == Some(&current_score.0);

	let (world_entity, ldtk_handle) = q_ldtk_world.single();

	let nb_levels = ldtk_asset.get(ldtk_handle).unwrap().project.levels.len();

	egui::CentralPanel::default().show(egui_ctx.ctx_mut(), |ui| {
		ui.vertical_centered(|ui| {
			let msg = match improved {
				true => "New best time!",
				false => "",
			};

			ui.label(msg);
			ui.heading(&current_score.0.to_string());
			ui.group(|ui| {
				for (name, score) in leaderboard.get_scores(level) {
					let style = ui.style_mut();

					if name == nickname.0 && score == current_score.0 {
						style.visuals.override_text_color = Some(Color32::RED);
					} else {
						style.visuals.override_text_color = None;
					};
					ui.label(format!("{}: {}", name, score));
				}
			});
			ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
				if ui.button("Next").clicked() || actions.just_pressed(Action::Jump) {
					*level_selection = LevelSelection::Index((level + 1) % nb_levels);
					next_app_state.set(AppState::Game);
				}
				if ui.button("Restart").clicked() || actions.just_pressed(Action::GroundPound) {
					commands.entity(world_entity).insert(Respawn);
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
