use std::{f32::consts::PI, time::Duration};

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        camera::RenderTarget,
        mesh::VertexAttributeValues,
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    },
    time::common_conditions::on_timer,
    window::{Monitor, PrimaryMonitor},
};
use bevy_prng::WyRand;
use bevy_rand::prelude::Entropy;
use rand_core::RngCore;

use crate::graphics::CAMERA_TRANFOMER;

use super::time::{TimePlugin, TimeSpan};

const CUBE_PIECE_SIZE: f32 = 1.0;
const CUBE_PIECE_OFFSET: f32 = CUBE_PIECE_SIZE * 1.1;
const CUBE_SIZE: f32 = CUBE_PIECE_SIZE * 3. + (CUBE_PIECE_OFFSET - CUBE_PIECE_SIZE) * 2.;
// 定义立方体的角
const LOCAL_CORNER: Vec3 = Vec3::new(CUBE_SIZE / 2., -CUBE_SIZE / 2., -CUBE_SIZE / 2.);

pub struct CubePlugin;

impl Plugin for CubePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TimePlugin)
            .insert_resource(RotationState {
                is_rotating: false,
                current_face: Face::Front,
                rotation_axis: Vec3::Z,
                rotation_direction: 0.5,
                progress: 0.,
            })
            .add_systems(Startup, setup)
            .add_systems(Update, auto_rotate.run_if(on_timer(Duration::from_secs(1))))
            .add_systems(Update, rotate_face);
    }
}

#[derive(Component)]
#[require(Visibility)]
struct Cube;

#[derive(Component, Debug)]
#[require(Mesh3d)]
struct CubePiece {
    x: i32,
    y: i32,
    z: i32,
}

// 旋转面枚举
#[derive(PartialEq, Copy, Clone, Debug)]
enum Face {
    Front,
    Back,
    Left,
    Right,
    Up,
    Down,
}

// 旋转状态资源
#[derive(Resource)]
struct RotationState {
    is_rotating: bool,
    current_face: Face,
    rotation_axis: Vec3,
    rotation_direction: f32,
    progress: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    monitor: Single<&Monitor, With<PrimaryMonitor>>,
) {
    let cube_texture = images.add(cube_texture());
    let texture_camera = commands
        .spawn((
            Camera2d,
            Camera {
                target: RenderTarget::Image(cube_texture.clone()),
                ..default()
            },
        ))
        .id();

    commands
        .spawn((
            Node {
                // Cover the whole image
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            TargetCamera(texture_camera),
        ))
        .with_children(|parent| {
            parent.spawn((
                TimeSpan,
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 50.,
                    ..default()
                },
                Transform::default().with_rotation(Quat::from_rotation_z(PI / 4.)),
            ));
        });

    let cube_handle = meshes.add(Cuboid::from_length(CUBE_SIZE));

    // This material has the texture that has been rendered.
    let time_material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(cube_texture),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // 创建水晶材质
    let glass_material = StandardMaterial {
        // 材质表面属性
        perceptual_roughness: 0.5, // 非常光滑的表面
        metallic: 0.8,             // 高金属感模拟冰面反射
        reflectance: 0.9,          // 增强反射强度

        // 透明度设置
        alpha_mode: AlphaMode::Blend, // 启用透明混合

        ..Default::default()
    };

    let viewport_size = Vec2::new(
        monitor.physical_width as f32,
        monitor.physical_height as f32,
    );
    let screen_pos = Vec2::new(viewport_size.x - 475., viewport_size.y - 435.);
    let cube_pos = screen_to_world(&CAMERA_TRANFOMER, screen_pos, viewport_size)
        .unwrap_or_default()
        .extend(0.);
    let cube_transform = Transform::from_translation(cube_pos);
    let cube_rotation = rotation_of_cube(&cube_transform, &CAMERA_TRANFOMER);
    let cube_transform = cube_transform.with_rotation(cube_rotation);

    commands
        .spawn((
            Mesh3d(cube_handle),
            MeshMaterial3d(time_material_handle),
            cube_transform,
            Cube,
            Entropy::<WyRand>::default(),
        ))
        .with_children(|commands| {
            let mut colorful_cube = Mesh::from(Cuboid::from_length(CUBE_PIECE_SIZE));
            if let Some(VertexAttributeValues::Float32x3(positions)) =
                colorful_cube.attribute(Mesh::ATTRIBUTE_POSITION)
            {
                let colors: Vec<[f32; 4]> = positions
                    .iter()
                    .map(|[r, g, b]| [(1. - *r) / 2., (1. - *g) / 2., (1. - *b) / 2., 0.3])
                    .collect();
                colorful_cube.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
            }
            let colorful_cube = meshes.add(colorful_cube);

            // 生成3x3x3魔方
            for x in -1..=1 {
                for y in -1..=1 {
                    for z in -1..=1 {
                        if x == 0 && y == 0 && z == 0 {
                            continue;
                        }

                        let mut glass_material = glass_material.clone();
                        glass_material.base_color = pos2color(x, y, z);
                        commands.spawn((
                            Mesh3d(colorful_cube.clone()),
                            MeshMaterial3d(materials.add(glass_material)),
                            Transform::from_xyz(
                                x as f32 * CUBE_PIECE_OFFSET,
                                y as f32 * CUBE_PIECE_OFFSET,
                                z as f32 * CUBE_PIECE_OFFSET,
                            ),
                            CubePiece { x, y, z },
                        ));
                    }
                }
            }
        });

    // 计算旋转后的对称轴方向（原局部坐标系中的对角线方向）
    let world_symmetry_axis = cube_rotation * -LOCAL_CORNER.normalize(); // 转换到世界坐标系

    // 设置光源沿对称轴方向偏移（距离根据正方体大小调整）
    let light_offset_distance = 5.0; // 光源距离中心的距离
    let light_position = cube_pos + world_symmetry_axis * light_offset_distance;
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(light_position),
    ));
}

fn screen_to_world(
    camera_transform: &Transform,
    screen_pos: Vec2,
    viewport_size: Vec2,
) -> Option<Vec2> {
    // 1. 视口逆变换：屏幕坐标转NDC
    let ndc_x = 1.0 - (screen_pos.x / viewport_size.x) * 2.0;
    let ndc_y = (screen_pos.y / viewport_size.y) * 2.0 - 1.0;

    // 2. 构造裁剪空间齐次坐标（假设深度为1.0，对应远平面）
    let clip_coords = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

    // 3. 逆投影变换：裁剪空间转视图空间
    let inverse_projection = Camera::default().clip_from_view().inverse();
    let view_homogeneous = inverse_projection * clip_coords;
    let view_coords = view_homogeneous.truncate() / view_homogeneous.w;

    // 4. 计算视图空间的射线方向并归一化
    let ray_dir_view = view_coords.normalize();

    // 5. 将射线方向转换到世界空间
    let ray_dir_world = camera_transform.rotation * ray_dir_view;

    // 6. 获取相机位置
    let camera_pos = camera_transform.translation;

    // 7. 计算射线与z=0平面的交点
    if ray_dir_world.z.abs() < 1e-6 {
        return None; // 射线平行于平面，无交点
    }

    let t = -camera_pos.z / ray_dir_world.z;
    let world_x = camera_pos.x + t * ray_dir_world.x;
    let world_y = camera_pos.y + t * ray_dir_world.y;

    Some(Vec2::new(world_x, world_y))
}

fn rotation_of_cube(cube_transform: &Transform, camera_transform: &Transform) -> Quat {
    let camera_pos = camera_transform.translation;
    let cube_pos = cube_transform.translation;

    // 计算正方体中心到相机的方向
    let target_dir = cube_pos - camera_pos;
    let target_dir_normalized = target_dir.normalize();

    // 正方体边长为1.0，角点初始局部坐标为(0.5, 0.5, 0.5)
    let initial_dir = LOCAL_CORNER.normalize(); // 初始方向向量

    // 计算旋转轴和角度
    let cross = initial_dir.cross(target_dir_normalized);
    let rotation_axis = cross.normalize();
    let cos_theta = initial_dir.dot(target_dir_normalized);
    let theta = cos_theta.acos();

    // 构造四元数并应用旋转
    Quat::from_axis_angle(rotation_axis, theta)
}

fn auto_rotate(
    mut rng: Single<&mut Entropy<WyRand>, With<Cube>>,
    mut rotation_state: ResMut<RotationState>,
) {
    if rotation_state.is_rotating {
        return;
    }

    let rotate_index = rng.next_u32() % 6;

    let (face, axis, dir) = match rotate_index {
        0 => (Face::Front, Vec3::Z, 1.),
        5 => (Face::Back, Vec3::Z, -1.),
        2 => (Face::Left, Vec3::X, -1.),
        4 => (Face::Right, Vec3::X, 1.),
        1 => (Face::Up, Vec3::Y, 1.),
        3 => (Face::Down, Vec3::Y, -1.),
        _ => return,
    };

    rotation_state.is_rotating = true;
    rotation_state.current_face = face;
    rotation_state.rotation_axis = axis;
    rotation_state.rotation_direction = dir;
    rotation_state.progress = 0.;
}

// 面旋转动画系统
fn rotate_face(
    time: Res<Time>,
    mut state: ResMut<RotationState>,
    mut query: Query<(&mut Transform, &mut CubePiece)>,
) {
    if !state.is_rotating {
        return;
    }

    let delta = time.delta_secs(); // 旋转速度
    state.progress += delta;

    // 计算旋转中心
    let center_position = CUBE_PIECE_SIZE * CUBE_PIECE_OFFSET;
    let center = match state.current_face {
        Face::Front => Vec3::new(0., 0., center_position),
        Face::Back => Vec3::new(0., 0., -center_position),
        Face::Left => Vec3::new(-center_position, 0., 0.),
        Face::Right => Vec3::new(center_position, 0., 0.),
        Face::Up => Vec3::new(0., center_position, 0.),
        Face::Down => Vec3::new(0., -center_position, 0.),
    };

    // 应用旋转动画
    for (mut transform, cube_piece) in query.iter_mut() {
        if is_piece_on_face(&cube_piece, state.current_face) {
            // 计算相对位置
            let rel_pos = transform.translation - center;

            // 创建旋转四元数
            let angle = delta * state.rotation_direction * PI / 2.;
            let rotation = Quat::from_axis_angle(state.rotation_axis, angle);

            // 更新位置和旋转
            transform.translation = center + rotation * rel_pos;
            transform.rotate(rotation);
        }
    }

    // 完成旋转后更新逻辑坐标
    if state.progress >= 1. {
        update_cube_positions(&mut query, state.current_face);
        state.is_rotating = false;
    }
}

fn cube_texture() -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: 512,
            height: 512,
            ..default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    image
}

// 判断方块是否在目标面上
fn is_piece_on_face(piece: &CubePiece, face: Face) -> bool {
    match face {
        Face::Front => piece.z == 1,
        Face::Back => piece.z == -1,
        Face::Left => piece.x == -1,
        Face::Right => piece.x == 1,
        Face::Up => piece.y == 1,
        Face::Down => piece.y == -1,
    }
}

// 更新立方体逻辑坐标
fn update_cube_positions(query: &mut Query<(&mut Transform, &mut CubePiece)>, face: Face) {
    for (mut transform, mut cube_piece) in query.iter_mut() {
        if is_piece_on_face(&cube_piece, face) {
            // 根据旋转面更新坐标
            let (x, y, z) = match face {
                Face::Front => (-cube_piece.y, cube_piece.x, cube_piece.z),
                Face::Back => (cube_piece.y, -cube_piece.x, cube_piece.z),
                Face::Left => (cube_piece.x, cube_piece.z, -cube_piece.y),
                Face::Right => (cube_piece.x, -cube_piece.z, cube_piece.y),
                Face::Up => (cube_piece.z, cube_piece.y, -cube_piece.x),
                Face::Down => (-cube_piece.z, cube_piece.y, cube_piece.x),
            };

            // 更新逻辑坐标
            cube_piece.x = x;
            cube_piece.y = y;
            cube_piece.z = z;

            // 重置物理位置
            transform.translation = Vec3::new(
                x as f32 * CUBE_PIECE_OFFSET,
                y as f32 * CUBE_PIECE_OFFSET,
                z as f32 * CUBE_PIECE_OFFSET,
            );
            transform.rotation = Quat::IDENTITY;
        }
    }
}

fn pos2color(x: i32, y: i32, z: i32) -> Color {
    let mapper = |i: i32| 0.3 * (i + 1) as f32 + 0.2;
    Color::srgba(mapper(x), mapper(y), mapper(z), 1.)
}
