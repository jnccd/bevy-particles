use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
    prelude::*,
    window::PrimaryWindow,
};

use crate::AppState;

pub struct ParticleScene<S: States> {
    pub state: S,
}

impl<S: States> Plugin for ParticleScene<S> {
    fn build(&self, app: &mut App) {
        app.init_resource::<FakeConstants>()
            .add_systems(
                OnEnter(self.state.clone()),
                (particles_setup, || println!("Particles Load!")),
            )
            .add_systems(Update, (particles_update, keyboard_input, particles_draw))
            .add_systems(OnExit(self.state.clone()), particles_teardown);
    }
}

const PARTICLE_SPATIAL_INTERVAL: f32 = 5.0;
const COLOR_BRIGHTNESS_MULT: f32 = 3.;
const PARTICLE_COLOR: Color = Color::srgb(
    0.3 * COLOR_BRIGHTNESS_MULT,
    0.8 * COLOR_BRIGHTNESS_MULT,
    1.0 * COLOR_BRIGHTNESS_MULT,
);
const GRAV_FORCE: f32 = 1400.;

#[derive(Component)]
pub struct ParticlesSceneCamera;

#[derive(Component, Debug)]
pub struct Particle {
    pos: Vec2,
    vel: Vec2,
}

#[derive(Resource, Default)]
pub struct FakeConstants {
    ORBIT_PULLVEC_ROT_MAT: Mat3,
}

const TRIANGLE_2D: Triangle2d = Triangle2d::new(Vec2::Y, Vec2::ZERO, Vec2::X);

fn particles_setup(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut consts: ResMut<FakeConstants>,
    _asset_server: Res<AssetServer>,
) {
    let window: &Window = window_query.get_single().unwrap();

    consts.ORBIT_PULLVEC_ROT_MAT = Mat3::from_angle(2.98);

    // Camera
    commands.spawn((
        Camera2d,
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom {
            intensity: 0.6,
            low_frequency_boost: 0.3,
            low_frequency_boost_curvature: 0.5,
            high_pass_frequency: 0.4,
            composite_mode: bevy::core_pipeline::bloom::BloomCompositeMode::Additive,
            prefilter: bevy::core_pipeline::bloom::BloomPrefilter {
                threshold: 0.,
                threshold_softness: 0.,
            },
            ..Default::default()
        },
        Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 1.0),
        ParticlesSceneCamera,
    ));

    let mesh = Triangle2d::new(Vec2::Y, Vec2::ZERO, Vec2::X);
    meshes.add(mesh);

    let mut particles_vec: Vec<Particle> = vec![];
    for x in 0..(window.width() / PARTICLE_SPATIAL_INTERVAL) as i32 {
        for y in 0..(window.height() / PARTICLE_SPATIAL_INTERVAL) as i32 {
            particles_vec.push(Particle {
                pos: Vec2::new(
                    x as f32 * PARTICLE_SPATIAL_INTERVAL,
                    y as f32 * PARTICLE_SPATIAL_INTERVAL,
                ),
                vel: Vec2::ZERO,
            });
        }
    }
    print!("#Particles: {}", particles_vec.len());
    commands.spawn_batch(particles_vec);
}

fn particles_update(
    mut particle_query: Query<&mut Particle>,
    consts: Res<FakeConstants>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window: &Window = window_query.get_single().unwrap();

    let mut cursor_pos_attract: Vec2 = Vec2::splat(-1.);
    let mut cursor_pos_repel: Vec2 = Vec2::splat(-1.);
    let mut cursor_pos_orbit: Vec2 = Vec2::splat(-1.);

    if let Ok((camera, camera_transform)) = camera_query.get_single() {
        if let Some(cursor_screen_position) = window.cursor_position() {
            if let Ok(cursor_world_position) =
                camera.viewport_to_world_2d(camera_transform, cursor_screen_position)
            {
                if mouse_buttons.pressed(MouseButton::Left) {
                    cursor_pos_attract = cursor_world_position;
                }

                if mouse_buttons.just_pressed(MouseButton::Right) {
                    cursor_pos_repel = cursor_world_position;
                }

                if mouse_buttons.pressed(MouseButton::Middle) {
                    cursor_pos_orbit = cursor_world_position;
                }
            }
        }
    }

    for mut particle in particle_query.iter_mut() {
        // Friction
        particle.vel *= 0.99;

        // Bounds
        if particle.pos.x < 0. || particle.pos.x > window.width() {
            particle.vel.x *= -1.0;
        }
        if particle.pos.y < 0. || particle.pos.y > window.height() {
            particle.vel.y *= -1.0;
        }

        // Cursor Attraction
        if cursor_pos_attract.x >= 0. {
            let diff: Vec2 = cursor_pos_attract - particle.pos;
            let diff_len_sq = diff.length_squared() / 4.5;
            let mut force_magnitude: f32 = GRAV_FORCE / diff_len_sq;
            if force_magnitude > 0.8 {
                force_magnitude = 0.8;
            }
            particle.vel += diff.normalize() * force_magnitude;
        }
        // Cursor Repel
        if cursor_pos_repel.x >= 0. {
            let diff: Vec2 = cursor_pos_repel - particle.pos;
            let diff_len_sq = diff.length_squared() / 8.;
            let mut force_magnitude: f32 = GRAV_FORCE / diff_len_sq;
            if force_magnitude > 1.8 {
                force_magnitude = 1.8;
            }
            force_magnitude *= 8.;
            particle.vel -= diff.normalize() * force_magnitude;
        }
        // Cursor Orbit
        if cursor_pos_orbit.x >= 0. {
            let mut diff: Vec2 = cursor_pos_orbit - particle.pos;
            diff = consts.ORBIT_PULLVEC_ROT_MAT.transform_vector2(diff);
            let diff_len = diff.length();
            let mut force_magnitude: f32 = GRAV_FORCE / diff_len / 8.;
            if force_magnitude > 0.8 {
                force_magnitude = 0.8;
            }
            particle.vel -= diff / diff_len * force_magnitude;
        }

        // Apply vel
        let vel_copy = Vec2::new(particle.vel.x, particle.vel.y);
        particle.pos += vel_copy;
    }
}

fn keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.pressed(KeyCode::Escape) {
        app_state.set(AppState::MainMenu);
    }
}

fn particles_draw(particle_query: Query<&Particle>, mut gizmos: Gizmos) {
    for particle in particle_query.iter() {
        gizmos.primitive_2d(
            &TRIANGLE_2D,
            Isometry2d {
                translation: particle.pos,
                ..Default::default()
            },
            PARTICLE_COLOR,
        );
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
