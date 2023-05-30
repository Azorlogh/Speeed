use std::time::Instant;

use bevy::{
	core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
	prelude::*,
	render::{camera::ScalingMode, view::ColorGrading},
};
use bevy_ecs_ldtk::{prelude::LdtkFields, LdtkLevel, LevelSelection, LevelSet, Respawn};
use bevy_egui::{egui, EguiContexts};
use bevy_rapier2d::prelude::CollisionEvent;

use crate::{
	input::Action,
	leaderboard::{CurrentScore, Leaderboard, Nickname, Score},
	level::{finish::Finish, LevelSize},
	player::Player,
	replay::ReplayRecording,
	states::{AppState, Exit},
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<Restart>()
			.add_systems((setup,).in_schedule(OnEnter(AppState::Game)))
			.add_system(exit.in_schedule(OnExit(AppState::Game)))
			.add_system(back_to_menu)
			.add_systems((restart, finish, ui).distributive_run_if(in_state(AppState::Game)));
		#[cfg(debug_assertions)]
		app.add_system(skip.run_if(in_state(AppState::Game)));
	}
}

/// When the current run started
#[derive(Resource)]
pub struct StartTime(pub Instant);

/// Allow pressing escape to go back
fn back_to_menu(mut next_app_state: ResMut<NextState<AppState>>, keys: Res<Input<KeyCode>>) {
	if keys.just_pressed(KeyCode::Escape) {
		next_app_state.set(AppState::Menu);
	}
}

fn exit(mut _commands: Commands) {}

fn setup(mut commands: Commands) {
	commands.insert_resource(StartTime(Instant::now()));

	let mut camera = Camera2dBundle::default();
	camera.projection.scaling_mode = ScalingMode::FixedVertical(2.0);
	camera.projection.scale = 2f32.powf(4.0);
	camera.transform.translation.z -= 100.0;
	camera.camera.hdr = true;
	camera.tonemapping = Tonemapping::TonyMcMapface;
	commands.spawn((
		camera,
		BloomSettings {
			intensity: 0.05,
			..default()
		},
		ColorGrading {
			exposure: 1.0,
			..default()
		},
		Exit(AppState::Game),
	));
}

fn ui(
	mut egui_ctx: EguiContexts,
	start_time: Res<StartTime>,
	nickname: Res<Nickname>,
	ldtk_levels: Res<Assets<LdtkLevel>>,
	q_level: Query<&Handle<LdtkLevel>>,
) {
	// Display the current time
	egui::Window::new("time")
		.movable(false)
		.collapsible(false)
		.resizable(false)
		.anchor(egui::Align2::CENTER_TOP, egui::Vec2::ZERO)
		.fixed_size(egui::Vec2::new(300.0, 100.0))
		.show(egui_ctx.ctx_mut(), |ui| {
			let score = Score(start_time.0.elapsed().as_millis() as u64);
			ui.label(format!("{score:.2}"));
		});

	// Display the level name
	if let Ok(level) = q_level.get_single().map(|h| ldtk_levels.get(h).unwrap()) {
		egui::Window::new("level-info")
			.movable(false)
			.collapsible(false)
			.resizable(false)
			.title_bar(false)
			.frame(egui::Frame::none().inner_margin(32.0))
			.anchor(egui::Align2::RIGHT_TOP, egui::Vec2::ZERO)
			.show(egui_ctx.ctx_mut(), |ui| {
				ui.label(format!("{}", level.level.get_string_field("name").unwrap()));
			});
	}

	// Display the current nickname
	egui::Window::new("player-info")
		.movable(false)
		.collapsible(false)
		.resizable(false)
		.title_bar(false)
		.frame(egui::Frame::none().inner_margin(32.0))
		.anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::ZERO)
		.show(egui_ctx.ctx_mut(), |ui| {
			ui.label(format!("{}", nickname.0));
		});
}

// simple helper to convert level grid coordinates to actual world coordinates
pub fn grid_to_world(level_size: &LevelSize, coord: IVec2) -> Vec2 {
	Vec2::new(
		coord.x as f32 + 0.5,
		level_size.get().y as f32 - coord.y as f32 - 0.5,
	)
}

/// Handle the player getting to the finish line
fn finish(
	mut commands: Commands,
	mut collision_events: EventReader<CollisionEvent>,
	mut q_player: Query<Entity, With<Player>>,
	mut q_finish: Query<Entity, With<Finish>>,
	start_time: Res<StartTime>,
	mut leaderboard: ResMut<Leaderboard>,
	level: Res<LevelSelection>,
	mut next_state: ResMut<NextState<AppState>>,
	nickname: Res<Nickname>,
	recording_replay: Res<ReplayRecording>,
) {
	let Ok(player_entity) = q_player.get_single_mut() else {
		return;
	};
	let Ok(finish_entity) = q_finish.get_single_mut() else {
		return;
	};
	for collision_event in collision_events.iter() {
		match collision_event {
			CollisionEvent::Started(e0, e1, _) => {
				if (*e0 == player_entity && *e1 == finish_entity)
					|| (*e1 == player_entity && *e0 == finish_entity)
				{
					let score = Score(start_time.0.elapsed().as_millis() as u64);
					if let LevelSelection::Index(level) = level.as_ref() {
						leaderboard.add_score(
							*level,
							&nickname.0,
							score,
							recording_replay.0.clone(),
						);
					}
					commands.insert_resource(CurrentScore(score));
					next_state.set(AppState::Leaderboard);
				}
			}
			_ => {}
		}
	}
}

/// Event to restart the level
pub struct Restart;

fn restart(
	mut commands: Commands,
	actions: Res<Input<Action>>,
	// mut q_player: Query<&mut Transform, With<Player>>,
	// mut q_spawn: Query<&ldtk::EntityInstance, With<Spawn>>,
	mut next_state: ResMut<NextState<AppState>>,
	mut ev_restart: EventReader<Restart>,
	q_ldtk_world: Query<Entity, With<LevelSet>>,
) {
	if ev_restart.iter().count() > 0 || actions.just_pressed(Action::Restart) {
		let world = q_ldtk_world.single();
		commands.entity(world).insert(Respawn);
		next_state.set(AppState::Game);
	}
}

// Skipping levels
#[cfg(debug_assertions)]
fn skip(
	mut commands: Commands,
	actions: Res<Input<Action>>,
	mut next_state: ResMut<NextState<AppState>>,
	q_ldtk_world: Query<(Entity, &Handle<bevy_ecs_ldtk::LdtkAsset>), With<LevelSet>>,
	ldtk_asset: Res<Assets<bevy_ecs_ldtk::LdtkAsset>>,
	mut level_selection: ResMut<LevelSelection>,
) {
	if actions.just_pressed(Action::Skip) {
		let LevelSelection::Index(level) = level_selection.clone() else {
			panic!("expected level index");
		};

		let (world_entity, ldtk_handle) = q_ldtk_world.single();

		let nb_levels = ldtk_asset.get(ldtk_handle).unwrap().project.levels.len();
		commands.entity(world_entity).insert(Respawn);
		*level_selection = LevelSelection::Index((level + 1) % nb_levels);
		next_state.set(AppState::Game);
	}
}
