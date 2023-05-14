use std::{error::Error, time::Duration};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{input::Action, states::AppState};

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(Leaderboard::load())
			.add_system(setup.in_schedule(OnEnter(AppState::Leaderboard)))
			.add_system(exit.in_schedule(OnExit(AppState::Leaderboard)))
			.add_system(menu_system.run_if(in_state(AppState::Leaderboard)));
	}
}

#[derive(Resource, Serialize, Deserialize)]
pub struct Leaderboard(Vec<u64>);

impl Leaderboard {
	pub fn load() -> Self {
		Self::try_load().unwrap_or(Leaderboard(vec![]))
	}

	pub fn try_load() -> Result<Self, Box<dyn Error>> {
		let s = std::fs::read_to_string("leaderboard.json")?;
		Ok(serde_json::from_str(&s)?)
	}

	pub fn save(&self) -> Result<(), Box<dyn Error>> {
		let s = serde_json::to_string(self)?;
		std::fs::write("leaderboard.json", s);
		Ok(())
	}

	pub fn add_score(&mut self, score: u64) {
		let pos = self.0.binary_search(&score).unwrap_or_else(|e| e);
		self.0.insert(pos, score);
		if let Err(e) = self.save() {
			warn!("failed to save leadeboard");
		}
	}
}

#[derive(Resource)]
pub struct CurrentScore(pub u64);

#[derive(Component)]
pub struct RestartButton;

fn menu_system(
	mut next_app_state: ResMut<NextState<AppState>>,
	actions: Res<Input<Action>>,
	q_restart: Query<&Interaction, With<RestartButton>>,
) {
	if *q_restart.single() == Interaction::Clicked || actions.just_pressed(Action::Jump) {
		next_app_state.set(AppState::Game);
	}
}

fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	score: Res<CurrentScore>,
	leaderboard: Res<Leaderboard>,
) {
	commands.spawn(Camera2dBundle::default());
	let font = asset_server.load("fonts/FiraSans-Bold.ttf");
	commands
		.spawn(NodeBundle {
			style: Style {
				size: Size::all(Val::Percent(100.0)),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				margin: UiRect::all(Val::Px(50.0)),
				flex_direction: FlexDirection::Column,
				..default()
			},
			..default()
		})
		.with_children(|builder| {
			// Title
			builder.spawn(TextBundle::from_section(
				"Well played!",
				TextStyle {
					font: asset_server.load("fonts/FiraSans-Bold.ttf"),
					font_size: 40.0,
					color: Color::BLACK,
				},
			));
			// Score
			builder.spawn(label(format_score(score.0), &font, 60.0, Color::RED));
			// Leaderboard
			builder
				.spawn(NodeBundle {
					style: Style {
						size: Size::all(Val::Percent(100.0)),
						align_items: AlignItems::Center,
						flex_direction: FlexDirection::Column,
						border: UiRect::all(Val::Px(2.0)),
						..default()
					},
					..default()
				})
				.with_children(|builder| {
					// Find our own score in the leaderboard to highlighing it
					let idx = leaderboard
						.0
						.iter()
						.enumerate()
						.filter(|(_, v)| score.0 == **v)
						.last()
						.unwrap()
						.0;
					for (i, s) in leaderboard.0.iter().enumerate() {
						let color = if i == idx { Color::RED } else { Color::BLACK };
						builder.spawn(label(format_score(*s), &font, 40.0, color));
					}
				});
			// Buttons
			builder
				.spawn((ButtonBundle::default(), RestartButton))
				.with_children(|builder| {
					builder.spawn(label("Restart".to_owned(), &font, 40.0, Color::BLACK));
				});
		});
}

fn label(text: String, font: &Handle<Font>, font_size: f32, color: Color) -> TextBundle {
	TextBundle::from_section(
		text,
		TextStyle {
			font: font.clone(),
			font_size,
			color,
		},
	)
}

fn format_score(score: u64) -> String {
	format!(
		"{}",
		humantime::format_duration(Duration::from_millis(score))
	)
}

fn exit(mut commands: Commands, q_nodes: Query<Entity, Or<(With<Node>, With<Camera>)>>) {
	for node in q_nodes.iter() {
		commands.entity(node).despawn();
	}
}
