use bevy::{asset::load_internal_binary_asset, prelude::*};
// use bevy_egui::egui::{FontData, FontDefinitions, FontFamily};
// use bevy_egui::EguiContexts;

/// 需要在DefaultPlugin之后加入
pub struct FontPlugin;

impl Plugin for FontPlugin {
    fn build(&self, app: &mut App) {
        load_internal_binary_asset!(
            app,
            Handle::default(),
            "../assets/fonts/LXGWWenKaiMono.ttf",
            |bytes: &[u8], _path: String| { Font::try_from_bytes(bytes.to_vec()).unwrap() }
        );
        // app.add_systems(Startup, set_egui_fonts);
    }
}

// fn set_egui_fonts(mut contexts: EguiContexts) {
//     let mut fonts = FontDefinitions::default();

//     // Install my own font (maybe supporting non-latin characters):
//     fonts.font_data.insert(
//         "lwgw".to_owned(),
//         FontData::from_static(include_bytes!("../assets/fonts/LXGWWenKaiMono.ttf")),
//     ); // .ttf and .otf supported

//     // Put my font first (highest priority):
//     fonts
//         .families
//         .get_mut(&FontFamily::Proportional)
//         .unwrap()
//         .insert(0, "lwgw".to_owned());

//     // Put my font as last fallback for monospace:
//     fonts
//         .families
//         .get_mut(&FontFamily::Monospace)
//         .unwrap()
//         .push("lwgw".to_owned());
//     contexts.ctx_mut().set_fonts(fonts);
// }
