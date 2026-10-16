//! Menus size to their content instead of the popup's default 600px area.

use egui_kittest::{kittest::Queryable as _, Harness};
use shadcn_egui::{menu_item, menu_separator};

#[test]
fn context_menu_fits_content() {
    let mut harness = Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_ui(|ui| {
        let r = ui.add(egui::Button::new("Target"));
        r.context_menu(|ui| {
            menu_item(ui, "Refresh", None);
            menu_separator(ui);
            menu_item(ui, "New console", Some("Ctrl+T"));
        });
    });
    harness.get_by_label("Target").click_secondary();
    harness.run_steps(4);
    let width = harness.get_by_label("New console").rect().width();
    assert!(width > 100.0 && width < 250.0, "menu row is {width}px wide");
    assert_eq!(harness.get_by_label("Refresh").rect().width(), width, "rows share the menu width");
}
