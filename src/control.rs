use std::{f32::consts::PI, time::Duration};

use crate::{
    objects::{spawn_ball, Ball, RespawnEvent},
    *,
};

#[derive(Deref, DerefMut, Resource, Default)]
pub struct ControlActive(bool);

pub fn control_ball(
    // q_ball: Query<Entity, With<Ball>>,
    mut respawn: EventReader<RespawnEvent>,
    // mut q_transform: Query<&mut Transform>,
    mut active: ResMut<ControlActive>,
) {
    if respawn.iter().count() == 0 {
        return;
    }

    // let ball = q_ball.single();
    // q_transform.get_mut(ball).unwrap().translation = Vec3::new(0.0, 1.0, TABLE.z / 2.0);
    **active = true;
}

#[derive(Resource)]
pub struct StoredVelocity {
    x_rot: f32,
    y_rot: f32,
    power: f32,
}

impl Default for StoredVelocity {
    fn default() -> Self {
        Self {
            x_rot: PI * 0.2,
            y_rot: 0.0,
            power: 5.0,
        }
    }
}

impl StoredVelocity {
    fn impulse(&self) -> ExternalImpulse {
        ExternalImpulse {
            impulse: Quat::from_euler(EulerRot::XYZ, self.x_rot, self.y_rot, 0.0)
                .mul_vec3(-Vec3::Z)
                * self.power
                * 0.005,
            ..default()
        }
    }
}

const BALL_POS: Vec3 = Vec3::new(0.0, 1.0, TABLE.z / 2.0);

pub fn ui(
    mut commands: Commands,
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
            stored.x_rot = stored.x_rot.clamp(0.0, PI / 2.0);
            ui.end_row();
            ui.label("Y rotation");
            ui.drag_angle(&mut stored.y_rot);
            stored.y_rot = stored.y_rot.clamp(-ANGLE, ANGLE);
            ui.end_row();
            ui.label("Power");
            ui.add(
                egui::DragValue::new(&mut stored.power)
                    .speed(0.1)
                    .clamp_range(2.0..=10.0),
            );
            ui.end_row();
        });
        if ui.button("Fire!").clicked() {
            *active = false;
            spawn_ball(
                BALL_POS,
                stored.impulse(),
                &mut commands,
                &mut meshes,
                &mut materials,
            );
        }
    });
}

#[derive(Deref, DerefMut)]
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

pub fn tracer(
    entities: Local<TracerEntities>,
    stored: Res<StoredVelocity>,
    rapier_config: Res<RapierConfiguration>,
    mut q_transform: Query<&mut Transform>,
) {
    let factor = match rapier_config.timestep_mode {
        TimestepMode::Fixed { dt, .. } => dt,
        TimestepMode::Variable { max_dt, .. } => max_dt,
        TimestepMode::Interpolated { dt, .. } => dt,
    };

    let per_step = 2;
    let force = stored.impulse().impulse;

    let mut pos = BALL_POS;
    let mut vel = force / factor * 4.0;
    for tracer in entities.iter() {
        for _ in 0..per_step {
            vel += rapier_config.gravity * factor;
            pos += vel * factor;
        }

        q_transform.get_mut(*tracer).unwrap().translation = pos;
    }
}

pub fn after_fire(
    time: Res<Time>,
    active: Res<ControlActive>,
    mut timer: Local<Option<Timer>>,
    mut last_active: Local<bool>,
) {
    if **active != *last_active {
        *last_active = **active;
        if **active {
            return;
        }

        *timer = Some(Timer::from_seconds(3.0, TimerMode::Once));
    }

    let Some(t) = &mut *timer else { return };

    t.tick(Duration::from_secs_f32(time.delta_seconds()));
    if t.finished() {
        *timer = None;
        dbg!("yee");
    }
}
