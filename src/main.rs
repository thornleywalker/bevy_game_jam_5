// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use avian3d::prelude::*;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                // Wasm builds will check for meta files (that don't exist) if this isn't set.
                // This causes errors and even panics in web builds on itch.
                // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
            PhysicsPlugins::default(),
        ))
        .insert_gizmo_config(
            PhysicsGizmos {
                aabb_color: Some(Color::WHITE),
                collider_color: Some(Color::linear_rgb(1.0, 0.1, 0.3)),
                ..default()
            },
            GizmoConfig::default(),
        )
        .add_systems(Startup, (setup_world, setup_bodies, setup_player))
        .add_systems(
            Update,
            (update_unit_models, test_advance_team, player_movement),
        )
        .run();
}

pub struct UnitBody {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Resource)]
pub struct UnitBodies {
    rock: UnitBody,
    paper: UnitBody,
    scissors: UnitBody,
}

fn setup_bodies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let rock = UnitBody {
        mesh: meshes.add(Sphere::default().mesh().ico(5).unwrap()),
        material: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(1.0, 0.3, 0.25),
            ..default()
        }),
    };
    let paper = UnitBody {
        mesh: meshes.add(Cuboid::from_size(Vec3 {
            x: 0.85,
            y: 1.1,
            z: 0.2,
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.3, 1.0, 0.0),
            ..default()
        }),
    };
    let scissors = UnitBody {
        mesh: meshes.add(Cone::default()),
        material: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.376, 0.425, 1.0),
            ..default()
        }),
    };

    commands.insert_resource(UnitBodies {
        rock,
        paper,
        scissors,
    });
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _asset_server: Res<AssetServer>,
) {
    // lights
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 5000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(20.0, 15.0, 10.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    // // camera
    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_translation(Vec3::new(0.0, 1.0, -5.0))
    //         .looking_at(Vec3::ZERO, Vec3::Y),
    //     ..default()
    // });

    // ground
    let ground_size = Vec3::new(500.0, 10.0, 500.0);
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Cuboid::from_size(ground_size)),
            material: materials.add(StandardMaterial {
                base_color: Color::linear_rgb(0.5, 0.25, 0.0),
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(0.0, -ground_size.y / 2.0, 0.0)),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(ground_size.x, ground_size.y, ground_size.z),
    ));

    // reference object

    let ref_obj_size = Vec3::new(1.0, 1.0, 1.0);
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Cuboid::from_size(ref_obj_size)),
            material: materials.add(StandardMaterial {
                base_color: Color::linear_rgb(0.5, 0.0, 0.0),
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(3.0, 0.0, 7.0)),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(ref_obj_size.x, ref_obj_size.y, ref_obj_size.z),
    ));
}

#[derive(Component, Clone, Copy)]
pub enum Team {
    Rock,
    Paper,
    Scissors,
}
impl Team {
    pub fn prey(&self) -> Self {
        match self {
            Team::Rock => Team::Scissors,
            Team::Paper => Team::Rock,
            Team::Scissors => Team::Paper,
        }
    }
    pub fn predator(&self) -> Self {
        match self {
            Team::Rock => Team::Paper,
            Team::Paper => Team::Scissors,
            Team::Scissors => Team::Rock,
        }
    }
}

#[derive(Component)]
pub struct PlayerControl;

#[derive(Component)]
pub struct NewTeam(pub Team);

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((
            PlayerControl,
            MaterialMeshBundle {
                mesh: meshes.add(Capsule3d::new(0.25, 0.5)), // starting shape, will get changed a bunch
                material: materials.add(StandardMaterial {
                    base_color: Color::BLACK,
                    ..default()
                }),
                transform: Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::capsule(0.25, 0.5),
            ExternalForce::ZERO, // for physics based player control
            LockedAxes::new().lock_rotation_x().lock_rotation_z(),
            LinearDamping(5.0),
            AngularDamping(8.0),
        ))
        .with_children(|commands| {
            commands.spawn(Camera3dBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 1.0, -5.0))
                    .looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            });
        });
}

fn update_unit_models(
    mut commands: Commands,
    mut units: Query<(
        Entity,
        &NewTeam,
        &mut Handle<Mesh>,
        &mut Handle<StandardMaterial>,
    )>,
    unit_bodies: Res<UnitBodies>,
) {
    units
        .iter_mut()
        .for_each(|(id, NewTeam(new_team), mut mesh, mut material)| {
            let unit_body = match new_team {
                Team::Rock => &unit_bodies.rock,
                Team::Paper => &unit_bodies.paper,
                Team::Scissors => &unit_bodies.scissors,
            };
            *mesh = unit_body.mesh.clone();
            *material = unit_body.material.clone();

            commands.entity(id).insert(*new_team).remove::<NewTeam>();
        });
}

fn test_advance_team(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    player: Query<(Entity, Option<&Team>), With<PlayerControl>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        player.iter().for_each(|(id, team)| {
            commands
                .entity(id)
                .insert(NewTeam(team.unwrap_or(&Team::Paper).prey()));
        });
    }
}

fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut ExternalForce, &Transform), With<PlayerControl>>,
) {
    let mut movement_dir = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        movement_dir += Vec3::Z;
    }
    if keys.pressed(KeyCode::KeyS) {
        movement_dir += -Vec3::Z;
    }
    if keys.pressed(KeyCode::KeyA) {
        movement_dir += Vec3::X;
    }
    if keys.pressed(KeyCode::KeyD) {
        movement_dir += -Vec3::X;
    }

    let movement_dir = movement_dir.normalize_or_zero();

    let strength = 4.0;

    player.iter_mut().for_each(|(mut force, transform)| {
        let mut new_force = transform.rotation * movement_dir * strength;
        new_force.y = force.force().y;

        force.set_force(new_force);
    });
}
