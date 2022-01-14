pub mod footer;
pub mod menu_bar;

pub trait AppComponent {
    type Context;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui);
}
