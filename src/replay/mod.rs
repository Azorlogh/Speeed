use std::path::PathBuf;

use anyhow::Result;
use bevy::{prelude::*, render::texture::DEFAULT_IMAGE_HANDLE};
use bevy_ecs_ldtk::{LdtkAsset, LevelSelection, LevelSet};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
	game::StartTime,
	leaderboard::{Leaderboard, Nickname},
	player::{Player, SpawnPlayer, PLAYER_SIZE},
	states::{AppState, Exit},
};

pub struct ReplayPlugin;

impl Plugin for ReplayPlugin {
	fn build(&self, app: &mut App) {
		app.add_system(recording_start.in_schedule(OnEnter(AppState::Game)))
			.add_system(recording_run.run_if(in_state(AppState::Game)).in_schedule(CoreSchedule::FixedUpdate))
			.add_system(ghost_spawn.in_schedule(OnEnter(AppState::Game)))
			.add_system(ghost_playback.run_if(in_state(AppState::Game)))
			// .add_system(save_replay.in_schedule(OnEnter(AppState::Leaderboard)))
			;
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ReplayData(Vec<Vec2>);

#[derive(Serialize, Deserialize)]
pub struct Replay {
	nickname: String,
	timestamp: OffsetDateTime,
	level: usize, // todo: use uuid
	data: ReplayData,
}

#[derive(Resource)]
pub struct ReplayRecording(pub ReplayData);

fn recording_run(
	q_player: Query<&Transform, With<Player>>,
	mut ev_player_spawned: EventReader<SpawnPlayer>,
	mut replay: ResMut<ReplayRecording>,
) {
	if ev_player_spawned.iter().count() > 0 {
		*replay = ReplayRecording(ReplayData(vec![]));
	}
	for tr in q_player.iter() {
		replay.0 .0.push(tr.translation.truncate());
	}
}

pub fn recording_start(mut commands: Commands) {
	commands.insert_resource(ReplayRecording(ReplayData(vec![])));
}

pub fn ghost_spawn(
	mut commands: Commands,
	leaderboard: Res<Leaderboard>,
	level_selection: ResMut<LevelSelection>,
	nickname: Res<Nickname>,
) {
	let LevelSelection::Index(level) = level_selection.clone() else {
		panic!("expected level index");
	};

	if let Some((_, replay)) = leaderboard.0[level].get(&nickname.0) {
		let Some(first_position) = replay.0.first() else {
			return;
		};
		commands.spawn((
			Ghost(replay.clone()),
			Sprite {
				color: Color::WHITE,
				custom_size: Some(Vec2::splat(PLAYER_SIZE)),
				..default()
			},
			SpatialBundle::from_transform(Transform::from_translation(first_position.extend(0.5))),
			DEFAULT_IMAGE_HANDLE.typed::<Image>(),
			Exit(AppState::Game),
		));
	}
}

#[derive(Component)]
pub struct Ghost(ReplayData);

pub fn ghost_playback(
	mut q_ghost: Query<(&Ghost, &mut Transform, &mut Sprite)>,
	start_time: Res<StartTime>,
	q_player: Query<&Transform, (With<Player>, Without<Ghost>)>,
) {
	if let Ok((ghost, mut tr, mut sprite)) = q_ghost.get_single_mut() {
		let since_start = start_time.0.elapsed().as_secs_f32() * 60.0;

		let Some(pos_prev) = ghost.0 .0.get(since_start as usize) else {
			return;
		};
		let Some(pos_next) = ghost.0 .0.get(since_start as usize + 1) else {
			return;
		};

		let t = since_start.rem_euclid(1.0);

		let pos = *pos_prev * (1.0 - t) + *pos_next * t;
		tr.translation = pos.extend(tr.translation.z);

		if let Ok(player_tr) = q_player.get_single() {
			const MIN_DIST: f32 = 2.0;
			const MAX_DIST: f32 = 10.0;
			let alpha = ((pos.distance_squared(player_tr.translation.truncate()) - MIN_DIST)
				/ (MAX_DIST - MIN_DIST))
				.clamp(0.0, 1.0) * 0.2;
			sprite.color = sprite.color.with_a(alpha);
		}
	}
}

// pub fn save_replay(
// 	mut level_selection: ResMut<LevelSelection>,
// 	q_ldtk_world: Query<(Entity, &Handle<LdtkAsset>), With<LevelSet>>,
// 	ldtk_asset: Res<Assets<LdtkAsset>>,
// 	replay: Res<RecordingReplay>,
// ) {
// 	let LevelSelection::Index(level) = level_selection.clone() else {
// 		panic!("expected level index");
// 	};

// 	if let Err(e) = try_save(&Replay {
// 		timestamp: OffsetDateTime::now_utc(),
// 		level,
// 		data: replay.0.clone(),
// 	}) {
// 		warn!("failed to save replay: {e}");
// 	}
// }

// fn try_save(replay: &Replay) -> Result<()> {
// 	let s = serde_json::to_string_pretty(replay)?;
// 	let path = replay_path().join(format!(
// 		"{}-{}-{}",
// 		replay.nickname,
// 		replay.level,
// 		replay.timestamp.unix_timestamp()
// 	));
// 	std::fs::create_dir_all(path.parent().unwrap())?;
// 	std::fs::write(path, s)?;
// 	Ok(())
// }

// fn replay_path() -> PathBuf {
// 	directories::ProjectDirs::from("", "Azorlogh", "Speeed")
// 		.unwrap()
// 		.data_dir()
// 		.to_owned()
// }
