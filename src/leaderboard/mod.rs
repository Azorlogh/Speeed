///
/// The leaderboard menu at the end of a run.
///
use std::error::Error;

use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

use crate::states::{AppState, Exit};

mod ui;

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(Leaderboard::load())
			.add_event::<ui::UiMessage>()
			.add_system(setup.in_schedule(OnEnter(AppState::Leaderboard)))
			.add_system(exit.in_schedule(OnExit(AppState::Leaderboard)))
			.add_systems(
				(
					ui::leaderboard_shortcuts,
					ui::leaderboard_ui,
					ui::leaderboard_update,
				)
					.distributive_run_if(in_state(AppState::Leaderboard)),
			);
	}
}

#[derive(Clone, Serialize, Deserialize, Resource)]
pub struct Nickname(pub String);
impl Default for Nickname {
	fn default() -> Self {
		Self(String::from("Anonymous player"))
	}
}

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

fn exit(mut commands: Commands, q_nodes: Query<Entity, Or<(With<Node>, With<Camera>)>>) {
	for node in q_nodes.iter() {
		commands.entity(node).despawn();
	}
}
