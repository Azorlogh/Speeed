use bevy::prelude::*;
use bevy_ecs_ldtk::{LdtkAsset, LevelSelection, LevelSet, Respawn};
use bevy_iced::{
	iced::{
		self, alignment, color,
		widget::{container, text, Button, Column, Container, Row, Space},
		Alignment, Length,
	},
	IcedContext,
};

use super::{CurrentScore, Leaderboard, Nickname};
use crate::{input::Action, states::AppState};

#[derive(Clone)]
pub enum UiMessage {
	LevelNext,
	LevelRestart,
}

pub fn leaderboard_shortcuts(actions: Res<Input<Action>>, mut messages: EventWriter<UiMessage>) {
	if actions.just_pressed(Action::Jump) {
		messages.send(UiMessage::LevelNext);
	} else if actions.just_pressed(Action::Restart) {
		messages.send(UiMessage::LevelNext);
	}
}

pub fn leaderboard_ui(
	mut ctx: IcedContext<UiMessage>,
	current_score: Res<CurrentScore>,
	leaderboard: Res<Leaderboard>,
	level_selection: Res<LevelSelection>,
	nickname: Res<Nickname>,
) {
	let LevelSelection::Index(level) = level_selection.clone() else {
		panic!("expected level index");
	};

	let improved = leaderboard.0[level].get(&nickname.0) == Some(&current_score.0);

	let leaderboard = {
		let mut col = Column::new()
			.push("Leaderboard")
			.push(Space::new(0.0, 16.0));
		for (name, score) in leaderboard.get_scores(level) {
			let mut text = text(format!("{}: {}", name, score));
			if name == nickname.0 && score == current_score.0 {
				text = text.style(color!(0xFF0000));
			}
			col = col.push(text);
		}
		col
	};

	let button_row = Row::new()
		.spacing(8.0)
		.push(Button::new("Restart").on_press(UiMessage::LevelRestart))
		.push(Button::new("Next").on_press(UiMessage::LevelNext));

	let msg = match improved {
		true => "New best time!",
		false => "Well done!",
	};

	let main = Column::new()
		.max_width(600.0)
		.align_items(Alignment::Center)
		.spacing(16.0)
		.push(text(msg).size(24.0))
		.push(text(current_score.0.to_string()).size(36.0))
		.push(Container::new(leaderboard).padding(16.0).style(
			container_appearance
				as for<'a> fn(&'a bevy_iced::iced_wgpu::Theme) -> container::Appearance,
		))
		.push(button_row);

	ctx.display(
		Container::new(Container::new(main).padding(16.0).style(
			container_appearance
				as for<'a> fn(&'a bevy_iced::iced_wgpu::Theme) -> container::Appearance,
		))
		.width(Length::Fill)
		.height(Length::Fill)
		.align_x(alignment::Horizontal::Center)
		.align_y(alignment::Vertical::Center),
	);
}

pub fn container_appearance(theme: &bevy_iced::iced_wgpu::Theme) -> container::Appearance {
	iced::widget::container::Appearance {
		border_radius: 5.0,
		border_width: 1.0,
		border_color: theme.palette().text,
		..default()
	}
}

pub fn leaderboard_update(
	mut commands: Commands,
	mut messages: EventReader<UiMessage>,
	mut level_selection: ResMut<LevelSelection>,
	q_ldtk_world: Query<(Entity, &Handle<LdtkAsset>), With<LevelSet>>,
	ldtk_asset: Res<Assets<LdtkAsset>>,
	mut next_app_state: ResMut<NextState<AppState>>,
) {
	let LevelSelection::Index(level) = level_selection.clone() else {
		panic!("expected level index");
	};

	let (world_entity, ldtk_handle) = q_ldtk_world.single();
	let nb_levels = ldtk_asset.get(ldtk_handle).unwrap().project.levels.len();

	for msg in messages.iter() {
		match msg {
			UiMessage::LevelNext => {
				*level_selection = LevelSelection::Index((level + 1) % nb_levels);
				next_app_state.set(AppState::Game);
			}
			UiMessage::LevelRestart => {
				commands.entity(world_entity).insert(Respawn);
				next_app_state.set(AppState::Game);
			}
		}
	}
}
