use bevy::{gizmos, math::NormedVectorSpace, prelude::*, window::PrimaryWindow};

pub struct ParticleScene<S: States> {
    pub state: S,
}

impl<S: States> Plugin for ParticleScene<S> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(self.state.clone()),
            (particles_setup, || println!("Particles Load!")),
        )
        .add_systems(Update, (particles_update, particles_draw))
        .add_systems(OnExit(self.state.clone()), particles_teardown);
    }
}

const PARTICLE_SPATIAL_INTERVAL: f32 = 5.0;
const PARTICLE_COLOR: Color = Color::srgb(0.3, 0.8, 1.0);

#[derive(Component)]
pub struct ParticlesSceneCamera;

#[derive(Component, Debug)]
pub struct Particle {
    pos: Vec2,
    vel: Vec2,
}

fn particles_setup(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    _asset_server: Res<AssetServer>,
) {
    let window: &Window = window_query.get_single().unwrap();

    // Camera
    commands.spawn((
        Camera2d { ..default() },
        Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 1.0),
        ParticlesSceneCamera,
    ));

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
    commands.spawn_batch(particles_vec);
}

fn particles_update(
    mut particle_query: Query<&mut Particle>,
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
        if cursor_pos.x >= 0. {
            let diff: Vec2 = cursor_pos - particle.pos;
            let diff_len = diff.length();
            let mut forceMagnitude: f32 = 1300. / (diff_len * diff_len);
            if (forceMagnitude > 0.8) {
                forceMagnitude = 0.8;
            }
            particle.vel += diff / diff_len * forceMagnitude;
        }

        // Apply vel
        let vel_copy = Vec2::new(particle.vel.x, particle.vel.y);
        particle.pos += vel_copy;
    }
}

fn particles_draw(
    particle_query: Query<&Particle>,
    mut gizmos: Gizmos,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    for particle in particle_query.iter() {
        gizmos.rect_2d(
            Isometry2d {
                translation: particle.pos,
                ..Default::default()
            },
            Vec2::ONE,
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
