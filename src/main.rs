use bevy::{input::mouse::MouseMotion, prelude::*};
use std::f32::consts::PI;
use bevy::ecs::system::ParamSet;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (player_movement, mouse_look,
            celestial_orbits,
            rotate_sun))
        .run();
}

#[derive(Component)]
struct Player {
    movement_speed: f32,
    rotation_speed: f32,
}

#[derive(Component)]
struct Velocity(Vec3); // 速度矢量

#[derive(Component)]
struct Star;

#[derive(Component)]
struct CelestialBody {
    mass: f32,          // 天体质量
    radius: f32,        // 天体半径
    is_star: bool,      // 是否是恒星（中心天体）
    rotation_speed: f32, // 自转速度
}

#[derive(Component)]
struct Orbit {
    semi_major_axis: f32, // 半长轴
    eccentricity: f32,    // 离心率
    inclination: f32,     // 轨道倾角（弧度）
    argument_of_periapsis: f32, // 近心点幅角（弧度）
    mean_anomaly: f32,    // 平近点角（弧度）
    orbital_period: f32,  // 轨道周期（秒）
}

const STAR_MASS: f32 = 1.0e8;
const GRAVITY_CONSTANT: f32 = 6.67430e-5; // 放大的万有引力常数
const TIME_SCALE: f32 = 1.0; // 时间因子
const MIN_DISTANCE: f32 = 1.0;  // 最小距离
const STAR_RADIUS: f32 = 10.0; // 太阳半径

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 光源
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 5000.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // 摄像机
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 100., 0.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        Player {
            movement_speed: 100.0,
            rotation_speed: 0.001,
        },
    ));

    // 创建太阳
    let star_mass = 1.0e8;
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(STAR_RADIUS))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.7, 0.1),
            emissive: Color::srgba(1.0, 0.6, 0.1, 1.0).into(),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        CelestialBody {
            mass: star_mass,
            radius: STAR_RADIUS,
            is_star: true,
            rotation_speed: 0.01,
        },
    ));

    // 创建行星
    let planets = [
        // 名称      质量   半径   轨道参数: (半长轴, 离心率, 倾角, 近心点幅角, 平近点角, 轨道周期) 颜色        自转速度
        ("水星", 0.3e3, 1.0, (20.0, 0.205, 7.0, 29.0, 0.0, 7.6), Color::srgb(0.7, 0.7, 0.7), 0.05),
        ("金星", 0.8e3, 1.5, (30.0, 0.007, 3.4, 55.0, 0.0, 19.4), Color::srgb(0.9, 0.7, 0.5), 0.02),
        ("地球", 1.0e3, 1.7, (40.0, 0.017, 0.0, 114.0, 0.0, 31.5), Color::srgb(0.2, 0.4, 0.8), 0.03),
        ("火星", 0.6e3, 1.3, (55.0, 0.093, 1.9, 286.0, 0.0, 59.3), Color::srgb(0.8, 0.4, 0.2), 0.04),
        ("木星", 10.0e3, 4.0, (90.0, 0.048, 1.3, 275.0, 0.0, 374.3), Color::srgb(0.8, 0.7, 0.5), 0.1),
        ("土星", 8.0e3, 3.5, (120.0, 0.056, 2.5, 336.0, 0.0, 929.0), Color::srgb(0.9, 0.8, 0.6), 0.09),
        ("天王星", 4.0e3, 2.5, (150.0, 0.046, 0.8, 99.0, 0.0, 2650.0), Color::srgb(0.5, 0.8, 0.9), 0.07),
        ("海王星", 3.8e3, 2.4, (180.0, 0.010, 1.8, 276.0, 0.0, 5200.0), Color::srgb(0.2, 0.4, 0.9), 0.06),
    ];

    for (name, mass, radius, (semi_major_axis, eccentricity, inclination, argument_of_periapsis, mean_anomaly, orbital_period), color, rotation_speed) in planets {
        // 计算初始位置 (开普勒轨道方程)
        let mean_anomaly:f32 = mean_anomaly;
        let true_anomaly:f32 = mean_anomaly + 2.0 * eccentricity * mean_anomaly.sin();
        let distance = semi_major_axis * (1.0 - eccentricity * eccentricity) / (1.0 + eccentricity * true_anomaly.cos());
        // 将角度转换为弧度
        let inclination_rad:f32 = inclination * PI / 180.0;
        let arg_periapsis_rad:f32 = argument_of_periapsis * PI / 180.0;
        let true_anomaly_rad:f32 = true_anomaly;
        
        // 计算位置 (在轨道平面内)
        let x = distance * (true_anomaly_rad.cos() * arg_periapsis_rad.cos() - 
                           true_anomaly_rad.sin() * arg_periapsis_rad.sin() * inclination_rad.cos());
        let y = distance * true_anomaly_rad.sin() * inclination_rad.sin();
        let z = distance * (true_anomaly_rad.cos() * arg_periapsis_rad.sin() + 
                           true_anomaly_rad.sin() * arg_periapsis_rad.cos() * inclination_rad.cos());
        
        // 计算轨道速度 (简化计算)
        let orbital_speed = (GRAVITY_CONSTANT * star_mass / semi_major_axis).sqrt();
        let velocity_direction = Vec3::new(-z, 0.0, x).normalize(); // 垂直于位置矢量
        let velocity = velocity_direction * orbital_speed;
        
        commands.spawn((
            Name::new(name.to_string()),
            Mesh3d(meshes.add(Sphere::new(radius * 1.0))),
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(x, y, z),
            Velocity(velocity),
            CelestialBody {
                mass: mass,
                radius: radius,
                is_star: false,
                rotation_speed: rotation_speed,
            },
            Orbit {
                semi_major_axis: semi_major_axis,
                eccentricity: eccentricity,
                inclination: inclination_rad,
                argument_of_periapsis: arg_periapsis_rad,
                mean_anomaly: mean_anomaly,
                orbital_period: orbital_period,
            },
        ));
    }
}

fn rotate_sun(
    time: Res<Time>,
    mut sun_query: Query<&mut Transform, With<Star>>, // 使用 Star 标记查询
) {
    for mut transform in sun_query.iter_mut() {
        transform.rotate_y(time.delta_secs() * 0.1);
    }
}


fn celestial_orbits(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Orbit, &CelestialBody)>,
) {
    let delta = time.delta_secs() * TIME_SCALE;
    
    for (mut transform, mut orbit, body) in query.iter_mut() {
        if body.is_star { continue; } // 跳过恒星
        
        // 更新平近点角
        orbit.mean_anomaly += (2.0 * PI * delta) / orbit.orbital_period;
        if orbit.mean_anomaly > 2.0 * PI {
            orbit.mean_anomaly -= 2.0 * PI;
        }
        
        // 使用开普勒方程计算偏近点角
        let mut eccentric_anomaly = orbit.mean_anomaly;
        for _ in 0..3 { // 迭代求解开普勒方程
            eccentric_anomaly = orbit.mean_anomaly + 
                orbit.eccentricity * eccentric_anomaly.sin();
        }
        
        // 计算真近点角
        let true_anomaly = 2.0 * ((1.0 + orbit.eccentricity).sqrt() * 
            (eccentric_anomaly / 2.0).sin()).atan2(
            ((1.0 - orbit.eccentricity).sqrt() * 
            (eccentric_anomaly / 2.0).cos())
        );
        
        // 计算距离
        let distance = orbit.semi_major_axis * 
            (1.0 - orbit.eccentricity * eccentric_anomaly.cos());
        
        // 计算位置 (在轨道平面内)
        let x = distance * true_anomaly.cos();
        let z = distance * true_anomaly.sin();
        
        // 应用轨道倾角和近心点幅角
        let inclination = orbit.inclination;
        let arg_periapsis = orbit.argument_of_periapsis;
        
        // 最终位置 (3D空间)
        let position = Vec3::new(
            x * arg_periapsis.cos() - z * arg_periapsis.sin() * inclination.cos(),
            x * arg_periapsis.sin() * inclination.sin() + z * inclination.sin(),
            x * arg_periapsis.sin() + z * arg_periapsis.cos() * inclination.cos()
        );
        
        // 更新行星位置
        transform.translation = position;
        
        // 行星自转
        transform.rotate_y(delta * body.rotation_speed);
    }
}


fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &Player)>,
    time: Res<Time>,
) {
    for (mut transform, player) in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction += *transform.forward();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction -= *transform.forward();
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction -= *transform.right();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += *transform.right();
        }
        if keyboard_input.pressed(KeyCode::Space) {
            direction += *transform.up();
        }
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            direction -= *transform.up();
        }

        if direction.length_squared() > 0.0 {
            direction = direction.normalize();
            transform.translation += direction * player.movement_speed * time.delta_secs();
        }
    }
}

fn mouse_look(
    mut motion_evr: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &Player)>,
) {
    let mut delta = Vec2::ZERO;
    for ev in motion_evr.read() {
        delta += ev.delta;
    }
    
    if delta.length_squared() > 0.0 {
        for (mut transform, player) in query.iter_mut() {
            // 左右旋转
            transform.rotate_y(-delta.x * player.rotation_speed);
            
            // 上下俯仰
            let rotation = transform.rotation;
            transform.rotation = Quat::from_euler(
                EulerRot::YXZ,
                rotation.to_euler(EulerRot::YXZ).0,
                (rotation.to_euler(EulerRot::YXZ).1 - delta.y * player.rotation_speed)
                    .clamp(-0.7, 0.7),
                0.0
            );
        }
    }
}
