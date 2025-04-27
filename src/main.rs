#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    prelude::*,
    window::{CursorOptions, PresentMode, WindowLevel},
};
use font::FontPlugin;
use graphics::GraphicsPlugin;

#[cfg(target_os = "macos")]
use bevy::window::CompositeAlphaMode;

mod font;
mod graphics;
mod ime;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    transparent: true,
                    decorations: false,
                    cursor_options: CursorOptions {
                        hit_test: false,
                        ..default()
                    },
                    present_mode: PresentMode::AutoNoVsync,
                    window_level: WindowLevel::AlwaysOnTop,
                    skip_taskbar: true,
                    #[cfg(target_os = "macos")]
                    composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                    ..default()
                }),
                ..default()
            }),
            FontPlugin,
        ))
        .add_plugins(GraphicsPlugin)
        .run();
}
