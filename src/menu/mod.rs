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

fn menu_system(
	mut next_app_state: ResMut<NextState<AppState>>,
	mut q_interaction: Query<&Interaction, With<Button>>,
) {
	let play = q_interaction.single_mut();
	if let Interaction::Clicked = *play {
		next_app_state.set(AppState::Game);
	}
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.spawn(Camera2dBundle::default());
	commands
		.spawn(NodeBundle {
			style: Style {
				size: Size::width(Val::Percent(100.0)),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				..default()
			},
			..default()
		})
		.with_children(|parent| {
			parent
				.spawn(ButtonBundle { ..default() })
				.with_children(|parent| {
					parent.spawn(TextBundle::from_section(
						"Button",
						TextStyle {
							font: asset_server.load("fonts/FiraSans-Bold.ttf"),
							font_size: 40.0,
							color: Color::rgb(0.9, 0.9, 0.9),
						},
					));
				});
		});
}

fn exit(mut commands: Commands, q_nodes: Query<(Entity, &Node)>) {
	for (node, _) in q_nodes.iter() {
		commands.entity(node).despawn();
	}
}
