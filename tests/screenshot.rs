//! Renders the gallery to `tests/snapshots/`.
//! Update the images with `UPDATE_SNAPSHOTS=1 cargo test`.

use egui_kittest::{kittest::Queryable as _, Harness};
use shadcn_egui::demo::Gallery;

fn harness(dark: bool) -> Harness<'static> {
    let mut gallery = Gallery::default();
    gallery.dark = dark;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1240.0, 1180.0))
        .build_ui(move |ui| gallery.ui(ui));
    harness.run_steps(4);
    harness
}

#[test]
fn gallery_light() {
    harness(false).snapshot("gallery_light");
}

#[test]
fn gallery_dark() {
    harness(true).snapshot("gallery_dark");
}

#[test]
fn dropdown_menu_open() {
    let mut h = harness(false);
    h.get_by_label("Open menu").click();
    h.run_steps(4);
    h.snapshot("dropdown_menu");
}

#[test]
fn select_open() {
    let mut h = harness(true);
    h.get_by_label("Select a fruit").click();
    h.run_steps(4);
    h.snapshot("select");
}

#[test]
fn dialog_open() {
    let mut h = harness(false);
    h.get_by_label("Edit profile").click();
    h.run_steps(8);
    h.snapshot("dialog");
}

#[test]
fn tree_open() {
    let mut h = harness(true);
    h.get_by_label("components").click();
    h.run_steps(8);
    h.get_by_label("button.rs").click();
    h.run_steps(4);
    h.snapshot("tree");
}

#[test]
fn data_table_sorted() {
    let mut h = harness(false);
    h.get_by_label("Amount").click();
    h.run_steps(2);
    h.get_by_label("Next").click();
    h.run_steps(4);
    h.snapshot("data_table");
}

#[test]
fn data_table_scroll() {
    let rows: Vec<Vec<String>> =
        (1..=20).map(|i| vec![format!("user{i:02}@example.com"), format!("${}.00", i * 37)]).collect();
    let mut selected = std::collections::HashSet::from([2]);
    let mut h = Harness::builder().with_size(egui::vec2(420.0, 420.0)).build_ui(move |ui| {
        shadcn_egui::Theme::light().install(ui.ctx());
        egui::Frame::new().fill(egui::Color32::WHITE).inner_margin(16).show(ui, |ui| {
            shadcn_egui::DataTable::new(
                "scroll",
                vec![shadcn_egui::Column::new("Email").weight(2.0).sortable(), shadcn_egui::Column::new("Amount").right()],
            )
            .filter("Filter emails...")
            .selection(&mut selected)
            .max_height(220.0)
            .show(ui, &rows);
        });
    });
    h.run_steps(2);
    h.event(egui::Event::PointerMoved(egui::pos2(200.0, 200.0)));
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, -150.0),
        modifiers: egui::Modifiers::NONE,
        phase: egui::TouchPhase::Move,
    });
    h.run_steps(30);
    h.snapshot("data_table_scroll");
}

#[test]
fn tree_scroll() {
    let mut selected = None;
    let mut h = Harness::builder().with_size(egui::vec2(320.0, 300.0)).build_ui(move |ui| {
        shadcn_egui::Theme::light().install(ui.ctx());
        egui::Frame::new().fill(egui::Color32::WHITE).inner_margin(16).show(ui, |ui| {
            shadcn_egui::Tree::new("big").default_open_depth(1).max_height(240.0).show(ui, &mut selected, |tree| {
                tree.folder(0, "src", |tree| {
                    for i in 1..=20 {
                        tree.leaf(i, &format!("file{i:02}.rs"));
                    }
                });
            });
        });
    });
    h.run_steps(2);
    h.get_by_label("src").click();
    h.run_steps(2);
    h.get_by_label("src").click(); // reopen: the first click closed it
    h.run_steps(8);
    for _ in 0..12 {
        h.key_press(egui::Key::ArrowDown);
        h.run_steps(2);
    }
    h.run_steps(20);
    h.snapshot("tree_scroll");
}
