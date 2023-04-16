use crate::*;

pub fn strobe_light(mut color: ResMut<ClearColor>, time: Res<Time>, mut q_light: Query<&mut PointLight>) {
    let cycle = 0.3;
    let i = (time.elapsed_seconds() / cycle) as usize;

    let c = if i % 2 == 0 {
        let hue = ((i / 2) as f32 * 20.0) % 360.0;
        Color::hsl(hue, 1.0, 0.98)
    } else {
        Color::rgb(1.0, 1.0, 1.0)
    };
    *color = ClearColor(c);
    // q_light.single_mut().color = c;
}

pub struct Light(Entity);

impl FromWorld for Light {
    fn from_world(world: &mut World) -> Self {
        let entity = world
            .spawn(PointLightBundle {
                point_light: PointLight {
                    color: Color::rgb(1.0, 1.0, 1.0),
                    intensity: 2000.0,
                    range: 100.0,
                    shadows_enabled: true,
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(0.0, 1.0, 10.0)),
                ..Default::default()
            })
            .id();
        Self(entity)
    }
}

// pub fn rotating_light(light: Local<Light>, mut q_transform: Query<&mut Transform>) {
//     q_transform
//         .get_mut(light.0)
//         .unwrap()
//         .rotate_around(Vec3::ZERO, Quat::from_rotation_y(0.01));
// }
