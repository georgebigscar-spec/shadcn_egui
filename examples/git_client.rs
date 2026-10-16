//! Run with `cargo run --example git_client`. Press Ctrl+K (⌘K) for the command palette.

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "shadcn_egui git client",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}

#[derive(Default)]
struct App(shadcn_egui::demo::GitDemo);

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }
}
