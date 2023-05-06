use bevy::prelude::*;

use crate::states::AppState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_system(setup.in_schedule(OnEnter(AppState::Menu)))
			.add_system(exit.in_schedule(OnExit(AppState::Menu)))
			.add_system(menu_system.run_if(in_state(AppState::Menu)));
	}
}

#[derive(Component)]
pub struct PlayButton;
#[derive(Component)]
pub struct EditButton;

fn menu_system(
	mut next_app_state: ResMut<NextState<AppState>>,
	q_btn_play: Query<&Interaction, With<PlayButton>>,
	q_btn_edit: Query<&Interaction, With<EditButton>>,
) {
	if let Interaction::Clicked = *q_btn_play.single() {
		next_app_state.set(AppState::Game);
	}
	if let Interaction::Clicked = *q_btn_edit.single() {
		next_app_state.set(AppState::Editor);
	}
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.spawn(Camera2dBundle::default());
	commands
		.spawn(NodeBundle {
			style: Style {
				size: Size::all(Val::Percent(100.0)),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				margin: UiRect::all(Val::Px(50.0)),
				..default()
			},
			..default()
		})
		.with_children(|builder| {
			builder
				.spawn(NodeBundle {
					style: Style {
						size: Size::all(Val::Percent(100.0)),
						align_items: AlignItems::Center,
						justify_content: JustifyContent::SpaceAround,
						flex_direction: FlexDirection::Column,
						..default()
					},
					..default()
				})
				.with_children(|builder| {
					builder
						.spawn((ButtonBundle::default(), PlayButton))
						.with_children(|builder| {
							builder.spawn(TextBundle::from_section(
								"Play",
								TextStyle {
									font: asset_server.load("fonts/FiraSans-Bold.ttf"),
									font_size: 40.0,
									color: Color::BLACK,
								},
							));
						});
					builder
						.spawn((ButtonBundle::default(), EditButton))
						.with_children(|builder| {
							builder.spawn(TextBundle::from_section(
								"Editor",
								TextStyle {
									font: asset_server.load("fonts/FiraSans-Bold.ttf"),
									font_size: 40.0,
									color: Color::BLACK,
								},
							));
						});
				});
		});
}

fn exit(mut commands: Commands, q_nodes: Query<Entity, Or<(With<Node>, With<Camera>)>>) {
	for node in q_nodes.iter() {
		commands.entity(node).despawn();
	}
}
