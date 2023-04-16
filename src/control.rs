use std::{f32::consts::PI, time::Duration};

use crate::{
    objects::{spawn_ball, Ball, BallBundle, Cup, RespawnEvent, BALL_RADIUS},
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

#[derive(Resource, Clone)]
pub struct StoredVelocity {
    x_rot: f32,
    y_rot: f32,
    power: f32,
}

impl Default for StoredVelocity {
    fn default() -> Self {
        Self {
            x_rot: PI * 0.27,
            y_rot: 0.0,
            power: 4.7,
        }
    }
}

impl StoredVelocity {
    fn impulse(&self, team: Team) -> ExternalImpulse {
        ExternalImpulse {
            impulse: Quat::from_euler(EulerRot::XYZ, self.x_rot * team.factor(), self.y_rot, 0.0)
                .mul_vec3(-Vec3::Z * team.factor())
                * self.power
                * 0.005,
            ..default()
        }
    }
}

fn ball_pos(team: Team) -> Vec3 {
    Vec3::new(0.0, 1.0, TABLE.z / 2.0 * team.factor())
}

pub fn ui(
    mut commands: Commands,
    mut active: ResMut<ControlActive>,
    mut ctx: EguiContexts,
    mut stored: ResMut<StoredVelocity>,
    mut local_stored: Local<[StoredVelocity; 2]>,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    ball_bundle: Local<BallBundle>,
    team: Res<Team>,
) {
    let active = &mut **active;

    if !*active {
        return;
    }
    let ctx = ctx.ctx_mut();

    egui::Window::new("Control").show(ctx, |ui| {
        egui::Grid::new("Grid").show(ui, |ui| {
            const ANGLE: f32 = PI / 8.0;
            ui.label(format!("Team {} is up!", *team as usize + 1));
            ui.end_row();
            ui.label("X rotation");
            ui.drag_angle(&mut local_stored[*team as usize].x_rot);
            local_stored[*team as usize].x_rot =
                local_stored[*team as usize].x_rot.clamp(0.0, PI / 2.0);
            ui.end_row();
            ui.label("Y rotation");
            ui.drag_angle(&mut local_stored[*team as usize].y_rot);
            local_stored[*team as usize].y_rot =
                local_stored[*team as usize].y_rot.clamp(-ANGLE, ANGLE);
            ui.end_row();
            ui.label("Power");
            ui.add(
                egui::DragValue::new(&mut local_stored[*team as usize].power)
                    .speed(0.1)
                    .clamp_range(2.0..=10.0),
            );
            ui.end_row();
            
            *stored = local_stored[*team as usize].clone();
        });
        if ui.button("Fire!").clicked() {
            *active = false;
            spawn_ball(
                ball_pos(*team),
                stored.impulse(*team),
                &mut commands,
                &*ball_bundle,
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
        for _ in 0..20 {
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
    active: Res<ControlActive>,
    team: Res<Team>,
) {
    if !**active || !stored.is_changed() {
        return;
    }

    let factor = match rapier_config.timestep_mode {
        TimestepMode::Fixed { dt, .. } => dt,
        TimestepMode::Variable { max_dt, .. } => max_dt,
        TimestepMode::Interpolated { dt, .. } => dt,
    };

    let per_step = 2;
    let force = stored.impulse(*team).impulse;

    let mut pos = ball_pos(*team);
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
    rapier_ctx: Res<RapierContext>,
    mut timer: Local<Option<Timer>>,
    mut last_active: Local<bool>,
    q_transform: Query<&Transform, With<Ball>>,
    has_sensor: Query<&Sensor>,
    q_parent: Query<&Parent>,
    mut respawn: EventWriter<RespawnEvent>,
    q_cup: Query<&Cup>,
) {
    if **active != *last_active {
        *last_active = **active;
        if **active {
            return;
        }

        *timer = Some(Timer::from_seconds(4.0, TimerMode::Once));
    }

    let Some(t) = &mut *timer else { return };

    t.tick(Duration::from_secs_f32(time.delta_seconds()));
    if t.finished() {
        *timer = None;
        let mut found = None;
        rapier_ctx.intersections_with_shape(
            q_transform.single().translation,
            Rot::IDENTITY,
            &Collider::ball(BALL_RADIUS * 0.02),
            QueryFilter::new().predicate(&|e| has_sensor.get(e).is_ok()),
            |e| {
                dbg!(e);
                found = Some(e);
                false
            },
        );

        if let Some(e) = found {
            let parent = q_parent.get(e).unwrap().get();
            respawn.send(RespawnEvent::HitCup(*q_cup.get(parent).unwrap()));
        } else {
            respawn.send(RespawnEvent::Missed);
        }
    }
}
