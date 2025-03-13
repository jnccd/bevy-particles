use bevy::{
    gizmos,
    math::{NormedVectorSpace, VectorSpace},
    prelude::*,
    render::render_resource::encase::private::Length,
    transform,
    window::PrimaryWindow,
};
use iyes_perf_ui::{entries::PerfUiFramerateEntries, prelude::*};

pub struct ParticleScene<S: States> {
    pub state: S,
}

impl<S: States> Plugin for ParticleScene<S> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(self.state.clone()),
            (particles_setup, || println!("Particles Load!")),
        )
        .add_systems(Update, (particles_update))
        .add_systems(OnExit(self.state.clone()), particles_teardown);
    }
}

const PARTICLE_SPATIAL_INTERVAL: f32 = 5.0;
const PARTICLE_COLOR: Color = Color::srgb(0.3, 0.8, 1.0);

#[derive(Component)]
pub struct ParticlesSceneCamera;

#[derive(Component, Debug)]
pub struct Particle {
    vel: Vec2,
}

fn particles_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window: &Window = window_query.get_single().unwrap();

    // Camera
    commands.spawn((
        Camera2d { ..default() },
        Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 1.0),
        ParticlesSceneCamera,
    ));

    let mesh = Triangle2d::new(Vec2::Y, Vec2::ZERO, Vec2::X);
    let img = asset_server.load("1.png");

    for x in 0..(window.width() / PARTICLE_SPATIAL_INTERVAL) as i32 {
        for y in 0..(window.height() / PARTICLE_SPATIAL_INTERVAL) as i32 {
            commands.spawn((
                Particle { vel: Vec2::ZERO },
                Sprite {
                    image: img.clone(),
                    ..default()
                },
                Transform::from_xyz(x as f32, y as f32, 0.0),
            ));
        }
    }
}

fn particles_update(
    mut particle_query: Query<(&mut Particle, &mut Transform)>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window: &Window = window_query.get_single().unwrap();

    let mut cursor_pos: Vec2 = Vec2::splat(-1.);
    if mouse_buttons.pressed(MouseButton::Left) {
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Some(cursor_screen_position) = window.cursor_position() {
                if let Ok(cursor_world_position) =
                    camera.viewport_to_world_2d(camera_transform, cursor_screen_position)
                {
                    cursor_pos = cursor_world_position;
                }
            }
        }
    }

    for (mut particle, mut particle_transform) in particle_query.iter_mut() {
        // Friction
        particle.vel *= 0.99;

        // Bounds
        if particle_transform.translation.x < 0.
            || particle_transform.translation.x > window.width()
        {
            particle.vel.x *= -1.0;
        }
        if particle_transform.translation.y < 0.
            || particle_transform.translation.y > window.height()
        {
            particle.vel.y *= -1.0;
        }

        // Cursor Attraction
        if cursor_pos.x >= 0. {
            let diff: Vec2 = cursor_pos - particle_transform.translation.xy();
            let diff_len = diff.length();
            let mut forceMagnitude: f32 = 1300. / (diff_len * diff_len);
            if (forceMagnitude > 0.8) {
                forceMagnitude = 0.8;
            }
            particle.vel += diff / diff_len * forceMagnitude;
        }

        // Apply vel
        particle_transform.translation.x += particle.vel.x;
        particle_transform.translation.y += particle.vel.y;
    }
}

pub fn particles_teardown(
    mut commands: Commands,
    particles_camera_query: Query<Entity, With<ParticlesSceneCamera>>,
    particle_query: Query<Entity, With<Particle>>,
) {
    if let Ok(particles_camera) = particles_camera_query.get_single() {
        commands.entity(particles_camera).despawn_recursive();
    }

    for particle in particle_query.iter() {
        commands.entity(particle).despawn_recursive();
    }
}
