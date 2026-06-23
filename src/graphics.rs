use bevy::{
    app::{Plugin, Startup},
    camera::Hdr,
    core_pipeline::tonemapping::Tonemapping,
    post_process::bloom::Bloom,
    prelude::*,
    window::{Monitor, PrimaryMonitor, PrimaryWindow, WindowResolution},
};
use cube::CubePlugin;

mod cube;
mod time;

pub struct GraphicsPlugin;

const CAMERA_TRANFOMER: Transform = Transform {
    translation: Vec3::new(15.0, 5.0, 15.0),
    rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 1.0),
    scale: Vec3::new(1.0, 1.0, 1.0),
};

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::NONE))
            .add_plugins(CubePlugin)
            .add_systems(Startup, (setup_window, setup_camera));
    }
}

fn setup_window(
    monitor: Single<&Monitor, With<PrimaryMonitor>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    // 完全等于屏幕大小会进入全屏模式，窗口背景变黑
    let width = monitor.physical_width - 1;
    let height = monitor.physical_height - 1;
    let scale_factor = monitor.scale_factor as f32;
    window.resolution =
        WindowResolution::new(width, height).with_scale_factor_override(scale_factor);
    window.position = WindowPosition::Centered(MonitorSelection::Primary);
}

fn setup_camera(mut commands: Commands) {
    // 添加相机
    commands.spawn((
        Camera3d::default(),
        Camera::default(),
        Hdr,
        Tonemapping::TonyMcMapface,
        CAMERA_TRANFOMER,
        Bloom::NATURAL,
    ));
}
