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
mod calendar;
mod command;
mod data_table;
mod diff;
mod form;
mod graph;
mod layout;
mod merge;
mod overlay;
mod resizable;
mod sidebar;
mod theme;
mod tree;

pub use basic::{h1, h2, h3, kbd, label, muted, separator, Avatar, Badge, BadgeVariant, Empty, Progress, Skeleton, Spinner};
pub use button::{Button, ButtonSize, ButtonVariant};
pub use calendar::{days_in_month, Calendar, Date, DatePicker, DateRange, DateRangePicker};
pub use command::{fuzzy_match, Combobox, Command, CommandResponse, CommandUi};
pub use data_table::{Column, DataTable, DataTableResponse};
pub use diff::{partial_patch, DiffView, DiffViewResponse};
pub use form::{button_group, radio_group, toggle_group, InputGroup, Checkbox, Input, RadioItem, Slider, Switch, Textarea, Toggle};
pub use graph::{CommitGraph, RefBadge, RefKind};
pub use layout::{breadcrumb, pagination, tabs, Accordion, Alert, Card, Collapsible, TabBar, TabBarResponse, Table};
pub use merge::{count_conflicts, resolve_conflicts, MergeView, MergeViewResponse, Resolution};
pub use overlay::{
    alert_dialog, context_menu, dropdown_menu, footer, hover_card, menu_checkbox, menu_item, menu_label,
    menu_separator, menubar, popover, toast, toast_error, Dialog, DialogResponse, MenubarUi, Select, Sheet,
    SheetSide, Toast, ToastId, ToastKind, Toaster, TooltipExt,
};
pub use resizable::Resizable;
pub use sidebar::{Sidebar, SidebarUi};
pub use theme::{mix, paint_focus_ring, Theme};
pub use tree::{git_status_color, Tree, TreeSelection, TreeUi};

pub mod demo;
