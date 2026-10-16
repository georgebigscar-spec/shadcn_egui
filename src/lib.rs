//! shadcn/ui-style components for egui.
//!
//! Call [`Theme::install`] once (for example in your app's `new`), then use
//! the widgets like any egui widget:
//!
//! ```ignore
//! use shadcn_egui::*;
//! Theme::light().install(ctx);
//! if ui.add(Button::outline("Cancel")).clicked() { /* ... */ }
//! ui.add(Input::new(&mut email).placeholder("Email"));
//! ```

mod basic;
mod button;
mod command;
mod data_table;
mod diff;
mod form;
mod graph;
mod layout;
mod overlay;
mod resizable;
mod sidebar;
mod theme;
mod tree;

pub use basic::{h1, h2, h3, kbd, label, muted, separator, Avatar, Badge, BadgeVariant, Empty, Progress, Skeleton, Spinner};
pub use button::{Button, ButtonSize, ButtonVariant};
pub use command::{fuzzy_match, Combobox, Command, CommandResponse, CommandUi};
pub use data_table::{Column, DataTable, DataTableResponse};
pub use diff::DiffView;
pub use form::{radio_group, toggle_group, Checkbox, Input, RadioItem, Slider, Switch, Textarea, Toggle};
pub use graph::CommitGraph;
pub use layout::{tabs, Accordion, Alert, Card, TabBar, TabBarResponse, Table};
pub use overlay::{
    alert_dialog, context_menu, dropdown_menu, footer, menu_checkbox, menu_item, menu_label, menu_separator,
    popover, toast, toast_error, Dialog, DialogResponse, Select, Toaster, TooltipExt,
};
pub use resizable::Resizable;
pub use sidebar::{Sidebar, SidebarUi};
pub use theme::{mix, paint_focus_ring, Theme};
pub use tree::{Tree, TreeSelection, TreeUi};

pub mod demo;
