use std::f32::consts::PI;

use crate::*;

const BALL_RADIUS: f32 = 5.0;

#[derive(Component)]
pub struct Object;

#[derive(Component)]
pub struct Ball;

pub struct RespawnEvent;

const CUP_OFFSET: f32 = 0.6;
const CUP_OFFSET_Y: f32 = 0.5;

pub fn spawn_objects(
    mut respawn: EventReader<RespawnEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
    mut q_objects: Query<Entity, With<Object>>,
) {
    if respawn.iter().count() == 0 {
        return;
    }

    for entity in q_objects.iter_mut() {
        commands.entity(entity).despawn();
    }

    // ball
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(
                shape::Icosphere {
                    radius: BALL_RADIUS,
                    subdivisions: 1,
                }
                .try_into()
                .unwrap(),
            ),
            transform: Transform::from_scale(Vec3::splat(0.02))
                .with_translation(Vec3::new(0.0, 2.0, 0.0)),
            material: materials.add(Color::rgb_u8(230, 138, 0).into()),
            ..default()
        })
        .insert((
            Ball,
            Object,
            Collider::ball(BALL_RADIUS),
            RigidBody::Dynamic,
            Restitution::new(RESTITUTION),
            Velocity::linear(Vec3::Z * 3.0)
        ));

    let height: f32 = f32::sqrt(3.0) / 2.0 * CUP_OFFSET;
    let positions = vec![
        Vec3::ZERO,
        Vec3::new(-CUP_OFFSET, 0.0, 0.0),
        Vec3::new(CUP_OFFSET, 0.0, 0.0),
        Vec3::new(CUP_OFFSET / 2.0, 0.0, height),
        Vec3::new(-CUP_OFFSET / 2.0, 0.0, height),
        Vec3::new(0.0, 0.0, height * 2.0),
    ];

    // cups
    for dir in [1.0, -1.0] {
        let center = Vec3::new(0.0, 0.05, (TABLE.z / 2.0 - CUP_OFFSET_Y) * -dir);
        for position in &positions {
            let mut p = position.clone();
            p.z *= dir;
            spawn_cup(center + p, &mut commands, &assets)
        }
    }
}

lazy_static! {
    static ref CUP_COLLIDER: Collider = {
        const CUP_RADIUS: f32 = 0.038;
        const CUP_CIRC: f32 = CUP_RADIUS * 2.0 * PI;
        const CUP_HEIGHT: f32 = 0.13 ;
        const ROTATE: f32 = -0.11;
        const CUP_THICKNESS: f32 = 0.005;
        const N: usize = 8;

        let mut compound = vec![];

        for i in 0..N {
            let angle = i as f32 / N as f32 * PI * 2.0;
            let x = angle.cos() * CUP_RADIUS;
            let z = angle.sin() * CUP_RADIUS;
            let y = CUP_HEIGHT / 2.0;
            let mut transform = Transform::from_xyz(x, y, z).with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                0.0,
                -angle - PI / 2.0,
                0.0,
            ));
            transform.rotate_local(Quat::from_rotation_x(ROTATE));
            let collider =
                Collider::cuboid(CUP_CIRC / N as f32 / 2.0, CUP_HEIGHT / 2.0, CUP_THICKNESS);
            compound.push((transform.translation, transform.rotation, collider));
        }
        // the buttom of the cup
        compound.push((
            Vec3::new(0.0,  0.0, 0.0),
            Quat::IDENTITY,
            Collider::cylinder(CUP_THICKNESS, CUP_RADIUS)
        ));

        Collider::compound(compound)
    };
}

fn spawn_cup(location: Vec3, commands: &mut Commands, assets: &Res<AssetServer>) {
    commands
        .spawn(
            (SceneBundle {
                transform: Transform::from_translation(location).with_scale(Vec3::splat(5.0)),
                scene: assets.load("scene.gltf#Scene0"),
                ..default()
            }),
        )
        .insert((Object, CUP_COLLIDER.clone(), RigidBody::Dynamic));
}
