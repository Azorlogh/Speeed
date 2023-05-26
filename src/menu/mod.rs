use bevy::prelude::*;
use bevy_ecs_ldtk::{prelude::LdtkFields, LdtkAsset, LevelSelection, LevelSet, Respawn};
use bevy_egui::{
	egui::{self, Align},
	EguiContexts,
};

use crate::{
	input::Action,
	leaderboard::{Leaderboard, Nickname},
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
	mut nickname: ResMut<Nickname>,
) {
	let LevelSelection::Index(level) = level_selection.clone() else {
		panic!("expected level index");
	};

	let (world, ldtk_handle) = q_ldtk_world.single();

	egui::CentralPanel::default().show(egui_ctx.ctx_mut(), |ui| {
		ui.text_edit_singleline(&mut nickname.0);
		egui::ScrollArea::horizontal().show(ui, |ui| {
			ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
				for (i, level) in ldtk_asset
					.get(ldtk_handle)
					.unwrap()
					.project
					.levels
					.iter()
					.enumerate()
				{
					let name = level.get_string_field("name").unwrap();

					ui.with_layout(egui::Layout::top_down(Align::LEFT), |ui| {
						ui.set_width(200.0);
						if ui
							.add(egui::Button::new(format!("Level {i}\n{name}")))
							.clicked()
						{
							*level_selection = LevelSelection::Index(i);
							next_app_state.set(AppState::Game);
							commands.entity(world).insert(Respawn);
						}
						ui.label("Best time: ");
						if let Some((name, score)) = leaderboard.get_scores(i).first() {
							ui.label(format!("{name}: {score}"));
						} else {
							ui.label("no score yet");
						}
					});
				}
			});
		});
	});

	// for i in 0..ldtk_asset.get(ldtk_handle).unwrap().project.levels.len() {
	// 	let level = &ldtk_asset.get(ldtk_handle).unwrap().project.levels[i];
	// 	let name = level.get_string_field("name").unwrap();
	// 	egui::Window::new(format!("level-{i}"))
	// 		.movable(false)
	// 		.collapsible(false)
	// 		.resizable(false)
	// 		.title_bar(false)
	// 		.show(egui_ctx.ctx_mut(), |ui| {
	// 			if ui
	// 				.add(
	// 					egui::Button::new(format!("Level {i}\n{name}"))
	// 						.min_size(egui::Vec2::new(100.0, 100.0)),
	// 				)
	// 				.clicked()
	// 			{
	// 				*level_selection = LevelSelection::Index(i);
	// 				next_app_state.set(AppState::Game);
	// 			}
	// 			ui.label("Best time: ");
	// 			if let Some(score) = leaderboard.0[i].first() {
	// 				ui.label(format!("{score}"));
	// 			} else {
	// 				ui.label("no score yet");
	// 			}
	// 		});
	// }
}
