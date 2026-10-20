//! `Tree::filter` shows matching rows and the folders that lead to them.

use egui_kittest::{kittest::Queryable as _, Harness};
use shadcn_egui::Tree;

fn tree(filter: &'static str) -> Harness<'static> {
    let mut selected: Option<&str> = None;
    let mut h = Harness::builder().with_size(egui::vec2(400.0, 600.0)).build_ui(move |ui| {
        Tree::new("files").filter(filter).show(ui, &mut selected, |tree| {
            tree.folder("src", "src", |tree| {
                tree.leaf("src/main.rs", "main.rs");
                tree.folder("src/widgets", "widgets", |tree| {
                    tree.leaf("src/widgets/button.rs", "button.rs");
                    tree.leaf("src/widgets/input.rs", "input.rs");
                });
            });
            tree.folder("docs", "docs", |tree| {
                tree.leaf("docs/guide.md", "guide.md");
            });
            tree.leaf_status("Cargo.toml", "Cargo.toml", "M");
        });
    });
    h.run();
    h
}

#[test]
fn filter_opens_folders_with_matches() {
    let h = tree("butn");
    for shown in ["src", "widgets", "button.rs"] {
        assert!(h.query_all_by_label(shown).next().is_some(), "{shown} shown");
    }
    for hidden in ["main.rs", "input.rs", "docs", "guide.md", "Cargo.toml"] {
        assert!(h.query_by_label(hidden).is_none(), "{hidden} hidden");
    }
}

#[test]
fn matching_folder_keeps_children() {
    let h = tree("docs");
    assert!(h.query_by_label("docs").is_some());
    assert!(h.query_by_label("src").is_none());
    // A matching folder is not forced open; its children appear once opened.
    assert!(h.query_by_label("guide.md").is_none());
}

#[test]
fn empty_filter_shows_everything() {
    let h = tree("");
    for shown in ["src", "docs", "Cargo.toml"] {
        assert!(h.query_all_by_label(shown).next().is_some(), "{shown} shown");
    }
}
