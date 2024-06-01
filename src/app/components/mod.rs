pub mod footer;
pub mod library_component;
pub mod menu_bar;
pub mod player_component;
pub mod playlist_table;
pub mod playlist_tabs;
pub mod scope_component;

pub trait AppComponent {
    type Context;

    fn add(ctx: &mut Self::Context, ui: &mut eframe::egui::Ui);
}
