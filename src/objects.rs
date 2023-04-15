use crate::*;

const BALL_RADIUS: f32 = 5.0;

#[derive(Component)]
pub struct Object;

#[derive(Component)]
pub struct Ball;

pub struct RespawnEvent;

const CUP_OFFSET: f32 = 0.2;
const CUP_OFFSET_Y: f32 = 1.2;

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
        ));

    // cups
    for dir in [1.0, -1.0] {
        let center = Vec3::new(0.0, 0.05, (TABLE.z / 2.0 - CUP_OFFSET) * dir);
        spawn_cup(center, &mut commands, &assets);
    }
}

fn spawn_cup(location: Vec3, commands: &mut Commands, assets: &Res<AssetServer>) {
    commands.spawn(SceneBundle {
        transform: Transform::from_translation(location).with_scale(Vec3::splat(6.0)),
        scene: assets.load("scene.gltf#Scene0"),
        ..default()
    });
}
