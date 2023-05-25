use bevy::prelude::*;
use bevy_ecs_ldtk::{LdtkAsset, LevelSelection, LevelSet};
use bevy_egui::{egui, EguiContexts};

use crate::{
	input::Action,
	leaderboard::{ Leaderboard},
	states::{AppState, Exit},
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
	fn build(&self, app: &mut App) {
		// app
		// 	.add_system(exit.in_schedule(OnExit(AppState::Menu)))
		app.add_system(setup.in_schedule(OnEnter(AppState::Menu)))
			.add_system(menu_ui.run_if(in_state(AppState::Menu)));
	}
}

fn setup(mut commands: Commands) {
	let mut camera = Camera2dBundle::default();
	camera.transform.translation.z = -10000.0;
	commands.spawn((camera, Exit(AppState::Menu)));
}

fn menu_ui(
	mut commands: Commands,
	leaderboard: Res<Leaderboard>,
	mut egui_ctx: EguiContexts,
	mut next_app_state: ResMut<NextState<AppState>>,
	mut level_selection: ResMut<LevelSelection>,
	q_ldtk_world: Query<(Entity, &Handle<LdtkAsset>), With<LevelSet>>,
	actions: Res<Input<Action>>,
	ldtk_asset: Res<Assets<LdtkAsset>>,
) {
	let LevelSelection::Index(level) = level_selection.clone() else {
		panic!("expected level index");
	};

	let (_, ldtk_handle) = q_ldtk_world.single();

	for i in 0..ldtk_asset.get(ldtk_handle).unwrap().project.levels.len() {
		egui::Window::new(format!("level-{i}"))
			.movable(false)
			.collapsible(false)
			.resizable(false)
			.title_bar(false)
			.show(egui_ctx.ctx_mut(), |ui| {
				if ui
					.add(egui::Button::new(format!("{i}")).min_size(egui::Vec2::new(100.0, 100.0)))
					.clicked()
				{
					*level_selection = LevelSelection::Index(i);
					next_app_state.set(AppState::Game);
				}
				ui.label("Best time: ");
				if let Some(score) = leaderboard.0[i].first() {
					ui.label(format!("{score}"));
				} else {
					ui.label("no score yet");
				}
			});
	}

	// let idx = leaderboard.0[level]
	// 	.iter()
	// 	.enumerate()
	// 	.filter(|(_, v)| score.0 == **v)
	// 	.map(|(i, _)| i)
	// 	.last();

	// egui::CentralPanel::default().show(egui_ctx.ctx_mut(), |ui| {
	// 	ui.vertical_centered(|ui| {
	// 		let msg = match idx.is_some() {
	// 			true => "Well played!",
	// 			false => "Better luck next time :)",
	// 		};
	// 		ui.label(msg);
	// 		ui.heading(&score.0.to_string());
	// 		ui.group(|ui| {
	// 			for (i, s) in leaderboard.0[level].iter().enumerate() {
	// 				let style = ui.style_mut();

	// 				if Some(i) == idx {
	// 					style.visuals.override_text_color = Some(Color32::RED);
	// 				} else {
	// 					style.visuals.override_text_color = None;
	// 				};
	// 				ui.label(&s.to_string());
	// 			}
	// 		});
	// 		ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
	// 			if ui.button("Next").clicked() || actions.just_pressed(Action::Jump) {
	// 				*level_selection = LevelSelection::Index((level + 1) % 7);
	// 				next_app_state.set(AppState::Game);
	// 			}
	// 			if ui.button("Restart").clicked() || actions.just_pressed(Action::GroundPound) {
	// 				let world = q_ldtk_world.single();
	// 				commands.entity(world).insert(Respawn);
	// 				next_app_state.set(AppState::Game);
	// 			}
	// 		});
	// 	});
	// });
}
