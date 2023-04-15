use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
// use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_rapier3d::prelude::*;

mod objects;
// mod cam;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(PanOrbitCameraPlugin)
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_event::<objects::RespawnEvent>()
        .add_startup_system(setup)
        .add_system(objects::spawn_objects)
        .add_system(disable_camera)
        .run();
}

struct BoolBuf(VecDeque<bool>);

impl Default for BoolBuf {
    fn default() -> Self {
        Self(vec![false; 4].into())
    }
}

fn disable_camera(
    mut ctx: EguiContexts,
    mut before: Local<BoolBuf>,
    mut q_pancam: Query<&mut PanOrbitCamera>,
) {
    let out = ctx.ctx_mut().wants_pointer_input();
    before.0.pop_front();
    before.0.push_back(out);
    let out = before.0.iter().any(|b| *b);

    q_pancam.single_mut().enabled = !out;
}

const RESTITUTION: f32 = 0.8;
const TABLE: Vec3 = Vec3::new(4.0, 0.1, 8.0);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut respawn: EventWriter<objects::RespawnEvent>,
) {
    let corner = TABLE / 2.0;
    let mesh = meshes.add(shape::Box::from_corners(corner, -corner).into());
    // table
    commands
        .spawn(PbrBundle {
            mesh,
            material: materials.add(Color::hsl(39.0, 1.0, 0.32).into()),
            ..default()
        })
        .insert((
            Collider::cuboid(corner.x, corner.y, corner.z),
            RigidBody::Fixed,
            Restitution::new(RESTITUTION),
        ));

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });
    // background
    commands.insert_resource(ClearColor(Color::hsl(0.0, 0.0, 0.8)));

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-0.0, 2.5, -7.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(), // FlyCamera::default(),
    ));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 8.0, 0.0),
        ..default()
    });

    respawn.send(objects::RespawnEvent)
}
