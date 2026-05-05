mod app;

pub use app::ProjectApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // Большее и удобное стартовое окно приложения
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([1024.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Project Manager",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();

            // Встраиваем шрифты в бинарник

            fonts.font_data.insert(
                "FiraCodeNerd".to_owned(),
                egui::FontData::from_static(include_bytes!(
                    "../assets/fonts/FiraCodeNerdFontPropo-Regular.ttf"
                ))
                .into(),
            );

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "FiraCodeNerd".to_owned());
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(ProjectApp::default()))
        }),
    )
}
