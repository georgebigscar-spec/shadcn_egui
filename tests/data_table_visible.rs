//! `DataTableResponse::visible` is the filtered, sorted row order.

use std::cell::RefCell;
use std::rc::Rc;

use egui_kittest::{kittest::Queryable as _, Harness};
use shadcn_egui::{Column, DataTable};

#[test]
fn visible_rows_follow_filter_and_sort() {
    let rows = vec![vec!["bob", "2"], vec!["alice", "10"], vec!["carol", "1"], vec!["bill", "3"]];
    let seen: Rc<RefCell<Vec<usize>>> = Rc::default();
    let out = seen.clone();
    let mut harness = Harness::builder().with_size(egui::vec2(800.0, 600.0)).build_ui(move |ui| {
        let columns = vec![Column::new("Name").sortable(), Column::new("Count").sortable().right()];
        let r = DataTable::new("t", columns).filter("Filter...").show(ui, &rows);
        *out.borrow_mut() = r.visible.to_vec();
    });
    harness.run();
    assert_eq!(*seen.borrow(), [0, 1, 2, 3]);

    // Numbers sort by value.
    harness.get_by_label("Count").click();
    harness.run();
    assert_eq!(*seen.borrow(), [2, 0, 3, 1]);

    harness.get_by_role(egui::accesskit::Role::TextInput).focus();
    harness.run();
    harness.get_by_role(egui::accesskit::Role::TextInput).type_text("b");
    harness.run();
    assert_eq!(*seen.borrow(), [0, 3]);
}
