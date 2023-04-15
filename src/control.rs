use std::f32::consts::PI;

use crate::{
    objects::{spawn_ball, Ball, RespawnEvent},
    *,
};

#[derive(Deref, DerefMut, Resource, Default)]
pub struct ControlActive(bool);

pub fn control_ball(
    q_ball: Query<Entity, With<Ball>>,
    mut respawn: EventReader<RespawnEvent>,
    mut q_transform: Query<&mut Transform>,
    mut active: ResMut<ControlActive>,
) {
    if respawn.iter().count() == 0 {
        return;
    }

    let ball = q_ball.single();
    q_transform.get_mut(ball).unwrap().translation = Vec3::new(0.0, 1.0, TABLE.z / 2.0);
    **active = true;
}

#[derive(Default, Resource)]
pub struct StoredVelocity {
    x_rot: f32,
    y_rot: f32,
    power: f32,
}

impl StoredVelocity {
    fn impulse(&self) -> ExternalImpulse {
        ExternalImpulse {
            impulse: Quat::from_euler(EulerRot::XYZ, self.x_rot, self.y_rot, 0.0)
                .mul_vec3(-Vec3::Z)
                * self.power,
            ..default()
        }
    }
}

pub fn ui(
    mut commands: Commands,
    q_ball: Query<Entity, With<Ball>>,
    mut active: ResMut<ControlActive>,
    mut ctx: EguiContexts,
    mut stored: ResMut<StoredVelocity>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let active = &mut **active;

    if !*active {
        return;
    }
    let ctx = ctx.ctx_mut();

    egui::Window::new("Control").show(ctx, |ui| {
        egui::Grid::new("Grid").show(ui, |ui| {
            const ANGLE: f32 = PI / 8.0;
            ui.label("X rotation");
            ui.drag_angle(&mut stored.x_rot);
            stored.x_rot = stored.x_rot.clamp(-ANGLE, ANGLE);
            ui.end_row();
            ui.label("Y rotation");
            ui.drag_angle(&mut stored.y_rot);
            stored.y_rot = stored.y_rot.clamp(-ANGLE, ANGLE);
            ui.end_row();
            ui.label("Power");
            ui.add(
                egui::DragValue::new(&mut stored.power)
                    .speed(0.1)
                    .clamp_range(5.0..=15.0),
            );
            ui.end_row();
        });
        if ui.button("Fire!").clicked() {
            *active = false;
            spawn_ball(
                Vec3::new(0.0, 1.0, TABLE.z / 2.0),
                stored.impulse(),
                &mut commands,
                &mut meshes,
                &mut materials,
            );
        }
    });
}

pub struct TracerEntities {
    entities: Vec<Entity>,
}

impl FromWorld for TracerEntities {
    fn from_world(world: &mut World) -> Self {
        let mut entities = Vec::new();
        let bundle = {
            let world = world.cell();
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            let mut materials = world
                .get_resource_mut::<Assets<StandardMaterial>>()
                .unwrap();
            PbrBundle {
                mesh: meshes.add(
                    shape::Icosphere {
                        radius: 2.0,
                        subdivisions: 1,
                    }
                    .try_into()
                    .unwrap(),
                ),
                transform: Transform::from_scale(Vec3::splat(0.02))
                    .with_translation(Vec3::new(0.0, 2.0, 0.0)),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgba_u8(255, 255, 255, 127),
                    unlit: true,
                    ..default()
                }),
                // visibility: Visibility::Hidden,
                ..default()
            }
        };
        for _ in 0..10 {
            entities.push(
                world
                    .spawn(bundle.clone())
                    .insert(Name::new("ball_tracer"))
                    .id(),
            )
        }
        Self { entities }
    }
}

pub fn tracer(entities: Local<TracerEntities>) {}
