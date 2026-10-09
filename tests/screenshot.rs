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
