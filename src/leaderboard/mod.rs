use std::{error::Error, time::Duration};

use bevy::prelude::*;
use bevy_ecs_ldtk::{LevelSelection, LevelSet, Respawn};
use bevy_egui::{
	egui::{self, Color32, Layout},
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
pub struct Leaderboard(Vec<Score>);

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
		std::fs::write("leaderboard.json", s)?;
		Ok(())
	}

	pub fn add_score(&mut self, score: Score) {
		let pos = self.0.binary_search(&score).unwrap_or_else(|e| e);
		self.0.insert(pos, score);
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

// fn menu_system(
// 	mut next_app_state: ResMut<NextState<AppState>>,
// 	actions: Res<Input<Action>>,
// 	q_restart: Query<&Interaction, With<RestartButton>>,
// ) {
// 	if *q_restart.single() == Interaction::Clicked || actions.just_pressed(Action::Jump) {
// 		next_app_state.set(AppState::Game);
// 	}
// }

fn setup(mut commands: Commands) {
	let mut camera = Camera2dBundle::default();
	camera.transform.translation.z = -10000.0;
	commands.spawn(camera);
}

fn leaderboard_ui(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	score: Res<CurrentScore>,
	leaderboard: Res<Leaderboard>,
	mut egui_ctx: EguiContexts,
	mut next_app_state: ResMut<NextState<AppState>>,
	mut level_selection: ResMut<LevelSelection>,
	q_ldtk_world: Query<Entity, With<LevelSet>>,
) {
	let idx = leaderboard
		.0
		.iter()
		.enumerate()
		.filter(|(_, v)| score.0 == **v)
		.last()
		.unwrap()
		.0;

	egui::CentralPanel::default().show(egui_ctx.ctx_mut(), |ui| {
		ui.vertical_centered(|ui| {
			ui.label("Well played!");
			ui.heading(&score.0.to_string());
			ui.group(|ui| {
				for (i, s) in leaderboard.0.iter().enumerate() {
					let style = ui.style_mut();

					if i == idx {
						style.visuals.override_text_color = Some(Color32::RED);
					} else {
						style.visuals.override_text_color = None;
					};
					ui.label(&s.to_string());
				}
			});
			ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui| {
				if ui.button("Restart").clicked() {
					let world = q_ldtk_world.single();
					commands.entity(world).insert(Respawn);
					next_app_state.set(AppState::Game);
				}
				if ui.button("Next").clicked() {
					let LevelSelection::Index(idx) = level_selection.as_mut() else {
						panic!("couldn't change level idx");
					};
					*idx += 1;
					// commands.entity(world).insert(Respawn);
					println!("{:?}", level_selection.as_ref());
					next_app_state.set(AppState::Game);
				}
			});
		});
	});

	// let font = asset_server.load("fonts/FiraSans-Bold.ttf");
	// commands
	// 	.spawn(NodeBundle {
	// 		style: Style {
	// 			size: Size::all(Val::Percent(100.0)),
	// 			align_items: AlignItems::Center,
	// 			justify_content: JustifyContent::Center,
	// 			margin: UiRect::all(Val::Px(50.0)),
	// 			flex_direction: FlexDirection::Column,
	// 			..default()
	// 		},
	// 		..default()
	// 	})
	// 	.with_children(|builder| {
	// 		// Title
	// 		builder.spawn(TextBundle::from_section(
	// 			"Well played!",
	// 			TextStyle {
	// 				font: asset_server.load("fonts/FiraSans-Bold.ttf"),
	// 				font_size: 40.0,
	// 				color: Color::BLACK,
	// 			},
	// 		));
	// 		// Score
	// 		builder.spawn(label(score.0.to_string(), &font, 60.0, Color::RED));
	// 		// Leaderboard
	// 		builder
	// 			.spawn(NodeBundle {
	// 				style: Style {
	// 					size: Size::all(Val::Percent(100.0)),
	// 					align_items: AlignItems::Center,
	// 					flex_direction: FlexDirection::Column,
	// 					border: UiRect::all(Val::Px(2.0)),
	// 					..default()
	// 				},
	// 				..default()
	// 			})
	// 			.with_children(|builder| {
	// 				// Find our own score in the leaderboard to highlighing it
	// 				let idx = leaderboard
	// 					.0
	// 					.iter()
	// 					.enumerate()
	// 					.filter(|(_, v)| score.0 == **v)
	// 					.last()
	// 					.unwrap()
	// 					.0;
	// 				for (i, s) in leaderboard.0.iter().enumerate() {
	// 					let color = if i == idx { Color::RED } else { Color::BLACK };
	// 					builder.spawn(label(s.to_string(), &font, 40.0, color));
	// 				}
	// 			});
	// 		// Buttons
	// 		builder
	// 			.spawn(NodeBundle {
	// 				style: Style {
	// 					size: Size::all(Val::Percent(100.0)),
	// 					align_items: AlignItems::Center,
	// 					flex_direction: FlexDirection::Row,
	// 					align_content: AlignContent::SpaceAround,
	// 					..default()
	// 				},
	// 				..default()
	// 			})
	// 			.with_children(|builder| {
	// 				builder
	// 					.spawn((ButtonBundle::default(), RestartButton))
	// 					.with_children(|builder| {
	// 						builder.spawn(label("Restart".to_owned(), &font, 40.0, Color::BLACK));
	// 					});
	// 				builder
	// 					.spawn((ButtonBundle::default(), NextButton))
	// 					.with_children(|builder| {
	// 						builder.spawn(label(
	// 							"Next Level".to_owned(),
	// 							&font,
	// 							40.0,
	// 							Color::BLACK,
	// 						));
	// 					});
	// 			});
	// 	});
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

fn exit(mut commands: Commands, q_nodes: Query<Entity, Or<(With<Node>, With<Camera>)>>) {
	for node in q_nodes.iter() {
		commands.entity(node).despawn();
	}
}
