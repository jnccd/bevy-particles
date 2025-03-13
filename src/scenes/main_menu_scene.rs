use crate::{AppState, TEXT_COLOR};
use bevy::{
    color::palettes::css::CRIMSON, prelude::*, text::cosmic_text::ttf_parser::Style,
    window::PrimaryWindow,
};

pub struct MainMenuScene<S: States> {
    pub state: S,
}

impl<S: States> Plugin for MainMenuScene<S> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(self.state.clone()),
            (main_menu_setup, || println!("Main Menu Load!")),
        )
        .add_systems(Update, (button_system, menu_action))
        .add_systems(OnExit(self.state.clone()), main_menu_teardown);
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component)]
pub struct MainMenuRoot;
#[derive(Component)]
pub struct MainMenuCamera;

#[derive(Component)]
enum MenuButtonAction {
    Play,
    // Settings,
    // BackToMainMenu,
    Quit,
}

#[derive(Component)]
struct SelectedOption;

fn main_menu_setup(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window: &Window = window_query.get_single().unwrap();

    // Camera
    commands.spawn((
        Camera2d { ..default() },
        Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 1.0),
        MainMenuCamera,
    ));

    // Common style for all buttons on the screen
    let button_node = Node {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_icon_node = Node {
        width: Val::Px(30.0),
        // This takes the icons out of the flexbox flow, to be positioned exactly
        position_type: PositionType::Absolute,
        // The icon will be close to the left border of the button
        left: Val::Px(10.0),
        ..default()
    };
    let button_text_font = TextFont {
        font_size: 33.0,
        ..default()
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            MainMenuRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Percent(0.),
                        padding: UiRect::all(Val::Px(90.)),
                        ..default()
                    },
                    BoxShadow {
                        color: Color::BLACK.with_alpha(0.9),
                        x_offset: Val::Percent(4.),
                        y_offset: Val::Percent(4.),
                        spread_radius: Val::Percent(3.),
                        blur_radius: Val::Percent(5.),
                    },
                    BorderRadius::all(Val::Percent(8.)),
                    BackgroundColor(CRIMSON.into()),
                ))
                .with_children(|parent| {
                    // Display the game name
                    parent.spawn((
                        Text::new("Bevy Particle System"),
                        TextFont {
                            font_size: 67.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                    ));

                    // Display three buttons for each action available from the main menu:
                    // - new game
                    // - settings
                    // - quit
                    parent
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            BorderRadius::all(Val::Percent(15.)),
                            MenuButtonAction::Play,
                        ))
                        .with_children(|parent| {
                            // let icon = asset_server.load("textures/Game Icons/right.png");
                            // parent.spawn((ImageNode::new(icon), button_icon_node.clone()));
                            parent.spawn((
                                Text::new("Play"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ));
                        });
                    // parent
                    //     .spawn((
                    //         Button,
                    //         button_node.clone(),
                    //         BackgroundColor(NORMAL_BUTTON),
                    //         MenuButtonAction::Settings,
                    //     ))
                    //     .with_children(|parent| {
                    //         // let icon = asset_server.load("textures/Game Icons/wrench.png");
                    //         // parent.spawn((ImageNode::new(icon), button_icon_node.clone()));
                    //         parent.spawn((
                    //             Text::new("Settings"),
                    //             button_text_font.clone(),
                    //             TextColor(TEXT_COLOR),
                    //         ));
                    //     });
                    parent
                        .spawn((
                            Button,
                            button_node,
                            BackgroundColor(NORMAL_BUTTON),
                            BorderRadius::all(Val::Percent(15.)),
                            MenuButtonAction::Quit,
                        ))
                        .with_children(|parent| {
                            // let icon = asset_server.load("textures/Game Icons/exitRight.png");
                            // parent.spawn((ImageNode::new(icon), button_icon_node));
                            parent.spawn((
                                Text::new("Quit"),
                                button_text_font,
                                TextColor(TEXT_COLOR),
                            ));
                        });
                });
        });
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color, selected) in &mut interaction_query {
        *background_color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_events.send(AppExit::Success);
                }
                MenuButtonAction::Play => {
                    app_state.set(AppState::InGame);
                }
            }
        }
    }
}

pub fn main_menu_teardown(
    mut commands: Commands,
    main_menu_root_query: Query<Entity, With<MainMenuRoot>>,
    main_menu_camera_query: Query<Entity, With<MainMenuCamera>>,
) {
    if let Ok(main_menu_entity) = main_menu_root_query.get_single() {
        commands.entity(main_menu_entity).despawn_recursive();
    }

    if let Ok(main_menu_entity) = main_menu_camera_query.get_single() {
        commands.entity(main_menu_entity).despawn_recursive();
    }
}
