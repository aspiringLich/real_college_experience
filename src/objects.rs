use std::f32::consts::PI;

use bevy::utils::HashSet;

use crate::*;

pub const BALL_RADIUS: f32 = 5.0;

#[derive(Component)]
pub struct Object;

#[derive(Component, Clone, Copy, Hash, PartialEq, Eq, Default, Debug)]
pub struct Cup(pub u8);

#[derive(Component)]
pub struct Ball;

pub enum RespawnEvent {
    Nothing,
    HitCup(Cup),
    Missed,
}

const CUP_OFFSET: f32 = 0.6;
const CUP_OFFSET_Y: f32 = 0.5;

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    #[default]
    Left,
    Right,
}

impl Team {
    pub fn factor(self) -> f32 {
        match self {
            Self::Left => 1.0,
            Self::Right => -1.0,
        }
    }
}

#[derive(Resource)]
pub struct BallBundle(PbrBundle);

impl FromWorld for BallBundle {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();

        Self(PbrBundle {
            mesh: meshes.add(
                shape::Icosphere {
                    radius: BALL_RADIUS,
                    subdivisions: 1,
                }
                .try_into()
                .unwrap(),
            ),
            transform: Transform::from_scale(Vec3::splat(0.02)),
            material: materials.add(Color::rgb_u8(230, 138, 0).into()),
            ..default()
        })
    }
}

const HEIGHT: f32 = 0.866025403784 * CUP_OFFSET;
const CUP_POSITIONS: &[Vec3] = &[
    Vec3::ZERO,
    Vec3::new(-CUP_OFFSET, 0.0, 0.0),
    Vec3::new(CUP_OFFSET, 0.0, 0.0),
    Vec3::new(CUP_OFFSET / 2.0, 0.0, HEIGHT),
    Vec3::new(-CUP_OFFSET / 2.0, 0.0, HEIGHT),
    Vec3::new(0.0, 0.0, HEIGHT * 2.0),
];

pub fn spawn_ball<T: Bundle>(
    translation: Vec3,
    bundle: T,
    commands: &mut Commands,
    ball: &BallBundle,
) {
    // ball
    commands.spawn(ball.0.clone()).insert((
        Transform::from_translation(translation).with_scale(Vec3::splat(0.02)),
        Ball,
        Object,
        Collider::ball(BALL_RADIUS),
        Restitution::new(RESTITUTION),
        RigidBody::Dynamic,
        bundle,
    ));
}

#[derive(Default, Resource)]
pub struct TakenCups {
    pub cups: [HashSet<Cup>; 2],
}

pub fn spawn_objects(
    mut respawn: EventReader<RespawnEvent>,
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
    mut q_objects: Query<Entity, With<Object>>,
    mut taken_cups: Local<TakenCups>,
    mut team: ResMut<Team>,
) {
    for event in respawn.iter() {
        for entity in q_objects.iter_mut() {
            commands.entity(entity).despawn_recursive();
        }

        match event {
            RespawnEvent::Nothing => {}
            RespawnEvent::HitCup(cup) => {
                taken_cups.cups[*team as usize].insert(*cup);
            }
            RespawnEvent::Missed => {
                match *team {
                    Team::Left => *team = Team::Right,
                    Team::Right => *team = Team::Left,
                }
            }
        }

        // cups
        dbg!(&taken_cups.cups);
        for (team, dir) in [1.0, -1.0].iter().enumerate() {
            let center = Vec3::new(0.0, 0.05, (TABLE.z / 2.0 - CUP_OFFSET_Y) * -dir);
            for (i, position) in CUP_POSITIONS.iter().enumerate() {
                if taken_cups.cups[team].contains(&Cup(i as u8)) {
                    continue;
                }
                let mut p = position.clone();
                p.z *= dir;
                spawn_cup(center + p, &mut commands, &assets, i as u8)
            }
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
            Vec3::new(0.0, 0.0, 0.0),
            Quat::IDENTITY,
            Collider::cylinder(CUP_THICKNESS, CUP_RADIUS)
        ));

        Collider::compound(compound)
    };
}

fn spawn_cup(location: Vec3, commands: &mut Commands, assets: &Res<AssetServer>, n: u8) {
    commands
        .spawn(SceneBundle {
            transform: Transform::from_translation(location).with_scale(Vec3::splat(5.0)),
            scene: assets.load("scene.gltf#Scene0"),
            ..default()
        })
        .insert((
            Object,
            CUP_COLLIDER.clone(),
            RigidBody::Dynamic,
            ColliderMassProperties::Density(1.0),
            Cup(n),
        ))
        .with_children(|parent| {
            parent.spawn((
                Transform::from_xyz(0.0, 0.2, 0.0),
                Object,
                Collider::cylinder(0.01, 0.15),
                Sensor,
            ));
        });
}
