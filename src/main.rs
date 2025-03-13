use bevy::prelude::*;
mod scenes;
use bevy::window::PrimaryWindow;
use bevy::window::WindowPlugin;
use bevy::window::WindowResized;
use bevy::window::WindowResolution;
use scenes::main_menu_scene::MainMenuScene;
use scenes::particle_scene::ParticleScene;

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum AppState {
    #[default]
    MainMenu,
    InGame,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(2560., 1440.).with_scale_factor_override(1.0),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Update, on_window_resize)
        .init_state::<AppState>()
        .add_plugins(MainMenuScene {
            state: AppState::MainMenu,
        })
        .add_plugins(ParticleScene {
            state: AppState::InGame,
        })
        .run();
}

fn on_window_resize(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut resize_events: EventReader<WindowResized>,
) {
    for e in resize_events.read() {
        if let Ok(mut window) = windows.get_single_mut() {
            window.resolution.set_scale_factor(1.);
        }
    }
}
