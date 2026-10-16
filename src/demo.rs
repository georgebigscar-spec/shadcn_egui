//! A gallery of every component, used by the `gallery` example and the screenshot test.

use egui::{Context, RichText, Ui};

use crate::*;

pub struct Gallery {
    pub dark: bool,
    email: String,
    password: String,
    bio: String,
    terms: bool,
    marketing: bool,
    airplane: bool,
    density: &'static str,
    volume: f32,
    bold: bool,
    align: usize,
    fruit: Option<usize>,
    tab: usize,
    show_status_bar: bool,
    dialog_open: bool,
    alert_open: bool,
    name: String,
    file: Option<&'static str>,
    payments: std::collections::HashSet<usize>,
}

impl Default for Gallery {
    fn default() -> Self {
        Self {
            dark: false,
            email: String::new(),
            password: String::new(),
            bio: String::new(),
            terms: true,
            marketing: false,
            airplane: true,
            density: "comfortable",
            volume: 40.0,
            bold: true,
            align: 0,
            fruit: None,
            tab: 0,
            show_status_bar: true,
            dialog_open: false,
            alert_open: false,
            name: "Pedro Duarte".into(),
            file: Some("src/tree.rs"),
            payments: [1].into(),
        }
    }
}

const PAYMENTS: [[&str; 2]; 6] = [
    ["ken99@example.com", "$316.00"],
    ["abe45@example.com", "$242.00"],
    ["monserrat44@example.com", "$837.00"],
    ["silas22@example.com", "$874.00"],
    ["carmella@example.com", "$721.00"],
    ["jason78@example.com", "$59.00"],
];

const FRUITS: [&str; 5] = ["Apple", "Banana", "Blueberry", "Grapes", "Pineapple"];

impl Gallery {
    /// Draws the whole gallery into the root `ui` (eframe's `App::ui`).
    pub fn ui(&mut self, ui: &mut Ui) {
        let ctx = &ui.ctx().clone();
        let theme = if self.dark { Theme::dark() } else { Theme::light() };
        if Theme::installed(ctx).as_ref() != Some(&theme) {
            theme.install(ctx);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Theme::get(ctx).background).inner_margin(32))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.content(ui));
            });

        self.dialogs(ctx);
        Toaster::show(ctx);
    }

    fn content(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            h2(ui, "shadcn_egui");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(Switch::new(&mut self.dark).label("Dark mode"));
            });
        });
        muted(ui, "shadcn/ui-style components built on egui 0.36.");
        ui.add_space(16.0);

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 24.0;
            ui.vertical(|ui| {
                ui.set_width(360.0);
                self.column_left(ui);
            });
            ui.vertical(|ui| {
                ui.set_width(360.0);
                self.column_middle(ui);
            });
            ui.vertical(|ui| {
                ui.set_width(360.0);
                self.column_right(ui);
            });
        });
    }

    fn column_left(&mut self, ui: &mut Ui) {
        Card::new()
            .title("Create an account")
            .description("Enter your email below to create your account")
            .width(360.0)
            .show(ui, |ui| {
                label(ui, "Email");
                ui.add(Input::new(&mut self.email).placeholder("m@example.com"));
                ui.add_space(4.0);
                label(ui, "Password");
                let short = !self.password.is_empty() && self.password.len() < 8;
                ui.add(Input::new(&mut self.password).password().invalid(short));
                if short {
                    ui.label(RichText::new("At least 8 characters").size(12.0).color(Theme::get(ui.ctx()).destructive));
                }
                ui.add_space(4.0);
                ui.add(Checkbox::new(&mut self.terms).label("Accept terms and conditions"));
                ui.add_space(8.0);
                if ui.add(Button::new("Create account").full_width()).clicked() {
                    toast(ui.ctx(), "Account created", Some("We sent a confirmation email."));
                }
                ui.add(Button::outline("Sign in with GitHub").full_width());
            });

        ui.add_space(16.0);
        label(ui, "Buttons");
        ui.horizontal_wrapped(|ui| {
            ui.add(Button::new("Default"));
            ui.add(Button::secondary("Secondary"));
            ui.add(Button::destructive("Destructive"));
            ui.add(Button::outline("Outline"));
            ui.add(Button::ghost("Ghost"));
            ui.add(Button::link("Link"));
            ui.add(Button::outline("+").size(ButtonSize::Icon)).tooltip("Add to library");
            ui.add(Button::new("Small").size(ButtonSize::Sm));
            ui.add_enabled(false, Button::new("Disabled"));
        });

        ui.add_space(12.0);
        label(ui, "Badges");
        ui.horizontal(|ui| {
            ui.add(Badge::new("Badge"));
            ui.add(Badge::new("Secondary").secondary());
            ui.add(Badge::new("Destructive").destructive());
            ui.add(Badge::new("Outline").outline());
        });

        ui.add_space(12.0);
        label(ui, "Avatar, Kbd, Skeleton");
        ui.horizontal(|ui| {
            ui.add(Avatar::new("CN"));
            ui.add(Avatar::new("GB").size(40.0));
            kbd(ui, "⌘");
            kbd(ui, "K");
            ui.add(Skeleton::circle(40.0));
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                ui.add(Skeleton::new([140.0, 14.0]));
                ui.add(Skeleton::new([100.0, 14.0]));
            });
        });

        ui.add_space(12.0);
        label(ui, "Tree");
        Tree::new("files").default_open_depth(1).show(ui, &mut self.file, |tree| {
            tree.folder("src", "src", |tree| {
                tree.folder("src/components", "components", |tree| {
                    tree.leaf("src/components/button.rs", "button.rs");
                    tree.leaf("src/components/card.rs", "card.rs");
                });
                tree.leaf("src/lib.rs", "lib.rs");
                tree.leaf("src/tree.rs", "tree.rs");
            });
            tree.folder("examples", "examples", |tree| {
                tree.leaf("examples/gallery.rs", "gallery.rs");
            });
            tree.leaf("Cargo.toml", "Cargo.toml");
        });
    }

    fn column_middle(&mut self, ui: &mut Ui) {
        tabs(ui, &mut self.tab, &["Account", "Password"]);
        ui.add_space(4.0);
        Card::new()
            .title(if self.tab == 0 { "Account" } else { "Password" })
            .description(if self.tab == 0 {
                "Make changes to your account here."
            } else {
                "Change your password here."
            })
            .width(360.0)
            .show(ui, |ui| {
                if self.tab == 0 {
                    label(ui, "Name");
                    ui.add(Input::new(&mut self.name));
                    label(ui, "Bio");
                    ui.add(Textarea::new(&mut self.bio).placeholder("Tell us a little bit about yourself"));
                } else {
                    label(ui, "New password");
                    ui.add(Input::new(&mut self.password).password());
                }
                ui.add_space(4.0);
                if ui.add(Button::new("Save changes")).clicked() {
                    toast(ui.ctx(), "Saved", None);
                }
            });

        ui.add_space(16.0);
        ui.add(Switch::new(&mut self.airplane).label("Airplane mode"));
        ui.add(Checkbox::new(&mut self.marketing).label("Send me marketing emails"));
        ui.add_space(4.0);
        radio_group(
            ui,
            &mut self.density,
            &[("default", "Default"), ("comfortable", "Comfortable"), ("compact", "Compact")],
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            label(ui, "Volume");
            ui.add(Slider::new(&mut self.volume, 0.0..=100.0).step(1.0).width(220.0));
            muted(ui, format!("{:.0}", self.volume));
        });
        ui.add(Progress::new(self.volume / 100.0));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add(Toggle::new(&mut self.bold, "B").outline()).tooltip("Toggle bold");
            toggle_group(ui, &mut self.align, &["Left", "Center", "Right"]);
        });
    }

    fn column_right(&mut self, ui: &mut Ui) {
        Alert::new("Heads up!")
            .icon("ℹ")
            .description("You can add components to your app using the cli.")
            .show(ui);
        ui.add_space(8.0);
        Alert::new("Unable to process your payment.")
            .icon("⚠")
            .description("Please verify your billing information and try again.")
            .destructive()
            .show(ui);

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add(Select::new(&mut self.fruit, &FRUITS).placeholder("Select a fruit"));
            let menu = ui.add(Button::outline("Open menu"));
            dropdown_menu(&menu, 220.0, |ui| {
                menu_label(ui, "My Account");
                menu_separator(ui);
                menu_item(ui, "Profile", Some("⌘P"));
                menu_item(ui, "Billing", Some("⌘B"));
                menu_item(ui, "Settings", Some("⌘S"));
                menu_separator(ui);
                menu_checkbox(ui, "Status bar", &mut self.show_status_bar);
                menu_separator(ui);
                if menu_item(ui, "Log out", Some("⌘Q")).clicked() {
                    toast_error(ui.ctx(), "Logged out", None);
                }
            });
        });
        ui.horizontal(|ui| {
            if ui.add(Button::outline("Edit profile")).clicked() {
                self.dialog_open = true;
            }
            if ui.add(Button::destructive("Delete")).clicked() {
                self.alert_open = true;
            }
            let pop = ui.add(Button::ghost("Popover"));
            popover(&pop, 260.0, |ui| {
                ui.label(RichText::new("Dimensions").strong());
                muted(ui, "Set the dimensions for the layer.");
            });
        });

        ui.add_space(12.0);
        let acc = Accordion::new("faq");
        acc.item(ui, "Is it accessible?", |ui| {
            ui.label("Yes. Widgets report AccessKit info and support keyboard focus.");
        });
        acc.item(ui, "Is it styled?", |ui| {
            ui.label("Yes. It comes with default styles that match shadcn/ui.");
        });
        acc.item(ui, "Is it animated?", |ui| {
            ui.label("Yes. Switches, accordions and progress bars animate.");
        });

        ui.add_space(12.0);
        Table::new(&["Invoice", "Status", "Method", "Amount"])
            .widths(&[1.0, 1.0, 1.3, 1.0])
            .right_align(&[3])
            .show(
                ui,
                &[
                    vec!["INV001", "Paid", "Credit Card", "$250.00"],
                    vec!["INV002", "Pending", "PayPal", "$150.00"],
                    vec!["INV003", "Unpaid", "Bank Transfer", "$350.00"],
                ],
            );

        ui.add_space(16.0);
        DataTable::new(
            "payments",
            vec![
                Column::new("Email").weight(2.0).sortable(),
                Column::new("Amount").sortable().right(),
            ],
        )
        .filter("Filter emails...")
        .selection(&mut self.payments)
        .page_size(4)
        .show(ui, &PAYMENTS.map(|r| r.to_vec()));
    }

    fn dialogs(&mut self, ctx: &Context) {
        if self.dialog_open {
            let r = Dialog::new("edit_profile", "Edit profile")
                .description("Make changes to your profile here. Click save when you're done.")
                .show(ctx, |ui| {
                    label(ui, "Name");
                    ui.add(Input::new(&mut self.name));
                    footer(ui, |ui| ui.add(Button::new("Save changes")).clicked())
                });
            if r.inner {
                toast(ctx, "Profile updated", Some(&self.name.clone()));
            }
            if r.inner || r.should_close {
                self.dialog_open = false;
            }
        }
        if self.alert_open {
            if let Some(ok) = alert_dialog(
                ctx,
                "delete_account",
                "Are you absolutely sure?",
                "This action cannot be undone. This will permanently delete your account.",
                "Continue",
            ) {
                self.alert_open = false;
                if ok {
                    toast_error(ctx, "Account deleted", None);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------

const BRANCHES: [&str; 6] = [
    "main",
    "develop",
    "feature/command-palette",
    "feature/resizable-panels",
    "fix/tree-scroll",
    "release/0.2",
];

const CHANGES: [(&str, &str); 4] =
    [("src/command.rs", "A"), ("src/graph.rs", "A"), ("src/overlay.rs", "M"), ("README.md", "M")];

/// (id, parents, subject, author), newest first.
const HISTORY: [(&str, &[&str], &str, &str); 10] = [
    ("4f1c2a9", &["9b0e7d1", "c3d8e45"], "Merge branch 'feature/git-widgets'", "George"),
    ("c3d8e45", &["5a7f210"], "Commit graph and diff view", "Ada"),
    ("9b0e7d1", &["e26b4c3"], "Add Command palette and Combobox", "George"),
    ("5a7f210", &["7d91f08"], "Sidebar, Spinner and Empty", "Ada"),
    ("e26b4c3", &["7d91f08"], "Resizable panel groups", "Linus"),
    ("7d91f08", &["1e4a6b2"], "Context menu on tree and table rows", "George"),
    ("1e4a6b2", &["b8c0d37"], "DataTable: custom cells and cached filter/sort", "Ada"),
    ("b8c0d37", &["0f3e9a1"], "Tree: row content and multi-select", "Linus"),
    ("0f3e9a1", &["6c2d5e8"], "Fix scrolling in Tree with max_height", "George"),
    ("6c2d5e8", &[], "Initial commit", "Ada"),
];

fn diff_for(path: &str) -> String {
    match path {
        "README.md" => "diff --git a/README.md b/README.md\n\
index 3b18e51..a9c2f04 100644\n\
--- a/README.md\n\
+++ b/README.md\n\
@@ -12,7 +12,8 @@ shadcn/ui-style components for egui\n \n ```sh\n cargo run --example gallery\n-cargo run --example git_client\n+cargo run --example git_client  # sidebar, graph, diff, tabs\n+UPDATE_SNAPSHOTS=1 cargo test\n ```\n \n ## Usage\n"
            .to_owned(),
        "src/overlay.rs" => "diff --git a/src/overlay.rs b/src/overlay.rs\n\
--- a/src/overlay.rs\n\
+++ b/src/overlay.rs\n\
@@ -40,9 +40,7 @@ impl Widget for Select<'_> {\n         response = response.on_hover_cursor(CursorIcon::PointingHand);\n \n-        let p = ui.painter();\n-        let fill = if t.dark { t.input.gamma_multiply(0.3) } else { Color32::TRANSPARENT };\n-        p.add(t.shadow_sm().as_shape(rect, t.radius_md()));\n+        match current {\n+            Some(s) => paint_select_trigger(ui, rect, s, t.foreground, &t),\n+            None => paint_select_trigger(ui, rect, &self.placeholder, t.muted_foreground, &t),\n+        }\n \n         let mut changed = false;\n"
            .to_owned(),
        _ => {
            let mut d = format!("diff --git a/{path} b/{path}\nnew file mode 100644\n--- /dev/null\n+++ b/{path}\n@@ -0,0 +1,24 @@\n");
            for line in [
                "//! A new module.",
                "",
                "use egui::{Color32, Pos2, Stroke, Ui};",
                "",
                "use crate::theme::Theme;",
                "",
                "pub struct Example {",
                "    lanes: usize,",
                "    pub lane_width: f32,",
                "}",
                "",
                "impl Example {",
                "    pub fn new() -> Self {",
                "        Self { lanes: 0, lane_width: 14.0 }",
                "    }",
                "",
                "    pub fn paint(&self, ui: &Ui) {",
                "        let t = Theme::get(ui.ctx());",
                "        let p = ui.painter();",
                "        p.circle_filled(Pos2::ZERO, 4.0, t.foreground);",
                "        p.line_segment([Pos2::ZERO, Pos2::new(0.0, 8.0)], Stroke::new(2.0, Color32::RED));",
                "    }",
                "}",
                "",
            ] {
                d.push('+');
                d.push_str(line);
                d.push('\n');
            }
            d
        }
    }
}

/// "Today" in the demo, so screenshots don't change with the date.
const DEMO_TODAY: Date = Date { year: 2026, month: 10, day: 16 };

/// Commit `i` (newest first) was made `i * 2` days before [`DEMO_TODAY`].
fn commit_date(i: usize) -> Date {
    DEMO_TODAY.add_days(-(i as i64) * 2)
}

/// What the sidebar selects.
#[derive(Clone, Debug, PartialEq)]
pub enum GitView {
    Changes,
    History,
    Branch(usize),
    Remote(&'static str),
}

/// A small git client: Sidebar, Resizable panels, Combobox, Command palette,
/// context menus, a commit graph, editor tabs and a diff view. Used by the
/// `git_client` example and the screenshot tests.
pub struct GitDemo {
    pub dark: bool,
    /// Widths of the sidebar | list | diff panels.
    pub sizes: [f32; 3],
    pub palette_open: bool,
    pub branch: Option<usize>,
    pub view: GitView,
    pub commit: usize,
    /// Files open in the diff tabs, and the active one.
    pub open_files: Vec<&'static str>,
    pub active_file: usize,
    pub split_diff: bool,
    pub details_open: bool,
    pub file_filter: String,
    /// History filter and page.
    pub dates: DateRange,
    pub page: usize,
    fetching_until: f64,
    file: Option<&'static str>,
    commits: Vec<Vec<String>>,
    graph: CommitGraph,
    diffs: std::collections::HashMap<&'static str, String>,
}

impl Default for GitDemo {
    fn default() -> Self {
        let commits = HISTORY
            .iter()
            .map(|(id, _, subject, author)| vec![String::new(), subject.to_string(), author.to_string(), id.to_string()])
            .collect();
        let graph = CommitGraph::new(HISTORY.iter().map(|(id, parents, _, _)| (*id, *parents)));
        let diffs = CHANGES.iter().map(|(p, _)| (*p, diff_for(p))).collect();
        Self {
            dark: false,
            sizes: [0.2, 0.42, 0.38],
            palette_open: false,
            branch: Some(0),
            view: GitView::History,
            commit: 0,
            open_files: vec!["src/command.rs", "src/overlay.rs", "README.md"],
            active_file: 1,
            split_diff: false,
            details_open: false,
            file_filter: String::new(),
            dates: DateRange::default(),
            page: 0,
            fetching_until: 0.0,
            file: Some("src/overlay.rs"),
            commits,
            graph,
            diffs,
        }
    }
}

impl GitDemo {
    pub fn ui(&mut self, ui: &mut Ui) {
        let ctx = &ui.ctx().clone();
        let theme = if self.dark { Theme::dark() } else { Theme::light() };
        if Theme::installed(ctx).as_ref() != Some(&theme) {
            theme.install(ctx);
        }
        let k = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::K);
        if ctx.input_mut(|i| i.consume_shortcut(&k)) {
            self.palette_open = !self.palette_open;
        }
        let t = Theme::get(ctx);
        let now = ctx.input(|i| i.time);

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(t.background).inner_margin(16))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    self.menu(ui);
                    ui.add(Combobox::new(&mut self.branch, &BRANCHES)
                        .placeholder("Select branch")
                        .search_placeholder("Search branch...")
                        .empty("No branch found.")
                        .width(240.0));
                    if button_group(ui, &["Fetch", "Pull", "Push"]).iter().any(|r| r.clicked()) {
                        self.fetching_until = now + 1.5;
                    }
                    if now < self.fetching_until {
                        ui.add(Spinner::new());
                        muted(ui, "Fetching origin…");
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(Button::outline("Search commands…  ⌘K").size(ButtonSize::Sm)).clicked() {
                            self.palette_open = true;
                        }
                    });
                });
                ui.add_space(12.0);
                egui::Frame::new()
                    .stroke(egui::Stroke::new(1.0, t.border))
                    .corner_radius(t.radius_lg())
                    .show(ui, |ui| {
                        let mut sizes = self.sizes;
                        Resizable::horizontal("git_panels", &mut sizes)
                            .min_size(180.0)
                            .with_handle()
                            .show(ui, |ui, panel| match panel {
                                0 => self.sidebar(ui),
                                1 if self.view == GitView::Changes => self.changes(ui),
                                1 => self.history(ui),
                                _ => self.details(ui),
                            });
                        self.sizes = sizes;
                    });
            });

        self.palette(ctx);
        self.commit_sheet(ctx);
        Toaster::show(ctx);
    }

    fn menu(&mut self, ui: &mut Ui) {
        menubar(ui, "git_menu", |mb| {
            mb.menu("Repository", |ui| {
                for (name, key) in [("Fetch", "⌘F"), ("Pull", "⌘P"), ("Push", "⇧⌘P")] {
                    if menu_item(ui, name, Some(key)).clicked() {
                        self.fetching_until = ui.input(|i| i.time) + 1.5;
                    }
                }
                menu_separator(ui);
                menu_item(ui, "Open in terminal", None);
            });
            mb.menu("Branch", |ui| {
                menu_item(ui, "New branch…", Some("⌘N"));
                menu_item(ui, "Merge into current…", None);
                menu_item(ui, "Rebase current…", None);
            });
            mb.menu("View", |ui| {
                menu_checkbox(ui, "Split diff", &mut self.split_diff);
                menu_separator(ui);
                if menu_item(ui, "Commit details", Some("⌘I")).clicked() {
                    self.details_open = true;
                }
                if menu_item(ui, "Command palette", Some("⌘K")).clicked() {
                    self.palette_open = true;
                }
            });
        });
    }

    fn commit_sheet(&mut self, ctx: &Context) {
        let (id, parents, subject, author) = HISTORY[self.commit];
        Sheet::new("git_commit_sheet", subject)
            .description(format!("{author} committed {id} on {}", commit_date(self.commit).long()))
            .show(ctx, &mut self.details_open, |ui| {
                let t = Theme::get(ui.ctx());
                ui.add_space(8.0);
                for (k, v) in [("Commit", id.to_owned()), ("Parents", parents.join(", ")), ("Author", author.to_owned())] {
                    ui.horizontal(|ui| {
                        ui.add_sized([80.0, 20.0], egui::Label::new(RichText::new(k).color(t.muted_foreground)));
                        ui.label(RichText::new(v).monospace());
                    });
                }
                ui.add_space(8.0);
                separator(ui);
                ui.add_space(8.0);
                label(ui, "Changed files");
                for (path, status) in CHANGES {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(status).monospace().color(t.muted_foreground));
                        ui.label(path);
                    });
                }
            });
    }

    fn sidebar(&mut self, ui: &mut Ui) {
        let t = Theme::get(ui.ctx());
        ui.painter().rect_filled(ui.max_rect(), egui::CornerRadius { nw: t.radius_lg().nw, sw: t.radius_lg().sw, ..Default::default() }, Sidebar::fill(&t));
        egui::Frame::new().inner_margin(egui::Margin::same(8)).show(ui, |ui| {
            Sidebar::new("git_nav").show(ui, &mut self.view, |sb| {
                sb.item(GitView::Changes, "Changes", Some(&CHANGES.len().to_string()));
                sb.item(GitView::History, "History", None);
                sb.group("Branches", |sb| {
                    for (i, b) in BRANCHES.iter().enumerate().take(4) {
                        let badge = (i == 2).then_some("+2");
                        sb.item(GitView::Branch(i), b, badge);
                    }
                });
                sb.group("Remotes", |sb| {
                    sb.item(GitView::Remote("origin"), "origin", None);
                    sb.item(GitView::Remote("upstream"), "upstream", None);
                });
            });
        });
    }

    fn changes(&mut self, ui: &mut Ui) {
        let t = Theme::get(ui.ctx());
        let before = self.file;
        egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
            ui.add(InputGroup::new(&mut self.file_filter).placeholder("Filter files...").search_icon().clearable());
            ui.add_space(8.0);
            let query = self.file_filter.clone();
            let visible: Vec<(&str, &str)> = CHANGES.iter().copied().filter(|(p, _)| fuzzy_match(&query, p)).collect();
            let staged: Vec<_> = visible.iter().filter(|(p, _)| *p == "README.md").copied().collect();
            let unstaged: Vec<_> = visible.iter().filter(|(p, _)| *p != "README.md").copied().collect();
            for (title, files) in [("Staged changes", staged), ("Changes", unstaged)] {
                Collapsible::new(("git_group", title), title).trailing(files.len().to_string()).show(ui, |ui| {
                    Tree::new(("git_files", title)).icons(true).show(ui, &mut self.file, |tree| {
                        for (path, status) in &files {
                            let name = path.rsplit('/').next().unwrap_or(path);
                            let row = tree.leaf_ui(*path, path, |ui| {
                                ui.label(name);
                                ui.label(RichText::new(*path).size(12.0).color(t.muted_foreground));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(*status).monospace().color(t.muted_foreground));
                                });
                            });
                            file_menu(&row, path);
                        }
                    });
                });
            }
        });
        if self.file != before {
            if let Some(f) = self.file {
                self.open(f);
            }
        }
    }

    fn open(&mut self, file: &'static str) {
        match self.open_files.iter().position(|f| *f == file) {
            Some(i) => self.active_file = i,
            None => {
                self.open_files.push(file);
                self.active_file = self.open_files.len() - 1;
            }
        }
    }

    fn history(&mut self, ui: &mut Ui) {
        const PAGE: usize = 6;
        let height = ui.available_height() - 24.0;
        egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
            ui.horizontal(|ui| {
                let picker = DateRangePicker::new("git_dates", &mut self.dates)
                    .placeholder("Filter by date")
                    .width(250.0)
                    .calendar(|c| c.today(DEMO_TODAY).max_date(DEMO_TODAY));
                if ui.add(picker).changed() {
                    self.page = 0;
                }
                if self.dates.start.is_some() && ui.add(Button::ghost("Reset").size(ButtonSize::Sm)).clicked() {
                    self.dates = DateRange::default();
                    self.page = 0;
                }
            });
            ui.add_space(8.0);

            // Rows in the date range; the graph only makes sense for the whole history.
            let filtered = self.dates.is_complete();
            let matching: Vec<usize> = (0..self.commits.len())
                .filter(|&i| !filtered || self.dates.contains(commit_date(i)))
                .collect();
            let page_count = matching.len().div_ceil(PAGE).max(1);
            self.page = self.page.min(page_count - 1);
            let shown: Vec<usize> = matching.iter().copied().skip(self.page * PAGE).take(PAGE).collect();
            let rows: Vec<Vec<String>> = shown.iter().map(|&i| self.commits[i].clone()).collect();

            let commits = &self.commits;
            let graph = &self.graph;
            let shown_ref = &shown;
            let mut details = None;
            let show_details = &mut details;
            let r = DataTable::new(
                "git_history",
                vec![
                    Column::new("").weight(0.55),
                    Column::new("Subject").weight(3.0),
                    Column::new("Author"),
                    Column::new("Hash").right(),
                ],
            )
            .row_height(32.0)
            .max_height(height - 140.0)
            .version(self.page as u64 * 1000 + matching.len() as u64)
            .cell_ui(|ui, row, col| {
                let i = shown_ref[row];
                match col {
                    0 => {
                        if !filtered {
                            graph.paint(ui, i);
                        }
                        true
                    }
                    3 => {
                        let t = Theme::get(ui.ctx());
                        let hash = ui.label(RichText::new(&commits[i][3]).monospace().color(t.muted_foreground));
                        hover_card(&hash, 300.0, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(Avatar::new(&commits[i][2][..1]).size(32.0));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(&commits[i][2]).color(t.foreground));
                                    muted(ui, format!("{} · {}", commits[i][3], commit_date(i).short()));
                                });
                            });
                            ui.label(RichText::new(&commits[i][1]).color(t.foreground));
                        });
                        true
                    }
                    _ => false,
                }
            })
            .row_context_menu(|ui, row| {
                let i = shown_ref[row];
                menu_label(ui, &commits[i][3]);
                menu_separator(ui);
                for action in ["Checkout", "Cherry-pick", "Revert"] {
                    if menu_item(ui, action, None).clicked() {
                        toast(ui.ctx(), format!("{action} {}", commits[i][3]), None);
                    }
                }
                menu_separator(ui);
                if menu_item(ui, "Show details", Some("⌘I")).clicked() {
                    *show_details = Some(i);
                }
                menu_item(ui, "Copy hash", Some("⌘C"));
            })
            .show(ui, &rows);
            if let Some(row) = r.clicked {
                self.commit = shown[row];
            }
            if let Some(i) = details {
                self.commit = i;
                self.details_open = true;
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                muted(ui, format!("{} commits", matching.len()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        pagination(ui, &mut self.page, page_count);
                    });
                });
            });
        });
    }

    fn details(&mut self, ui: &mut Ui) {
        let r = TabBar::new("git_tabs").show(ui, &mut self.open_files, &mut self.active_file, |f| {
            f.rsplit('/').next().unwrap_or(f).to_owned()
        });
        if let Some(f) = r.closed {
            if self.file == Some(f) {
                self.file = None;
            }
        }
        let Some(file) = self.open_files.get(self.active_file).copied() else {
            ui.add_space(40.0);
            Empty::new("No open files")
                .icon("±")
                .description("Pick a changed file to see its diff.")
                .show(ui, |ui| {
                    if ui.add(Button::outline("Open all changes").size(ButtonSize::Sm)).clicked() {
                        for (f, _) in CHANGES {
                            self.open(f);
                        }
                    }
                });
            return;
        };
        egui::Frame::new().inner_margin(egui::Margin::symmetric(12, 8)).show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut path: Vec<&str> = vec!["shadcn_egui"];
                path.extend(file.split('/'));
                if breadcrumb(ui, &path) == Some(0) {
                    self.view = GitView::Changes;
                }
                ui.add_space(8.0);
                let diff = &self.diffs[file];
                let count = |c: char| diff.lines().filter(|l| l.starts_with(c) && !l.starts_with("+++") && !l.starts_with("---")).count();
                let green = if Theme::get(ui.ctx()).dark { egui::Color32::from_rgb(74, 222, 128) } else { egui::Color32::from_rgb(22, 163, 74) };
                ui.label(RichText::new(format!("+{}", count('+'))).monospace().color(green));
                ui.label(RichText::new(format!("-{}", count('-'))).monospace().color(Theme::get(ui.ctx()).destructive));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut mode = self.split_diff as usize;
                    toggle_group(ui, &mut mode, &["Unified", "Split"]);
                    self.split_diff = mode == 1;
                });
            });
        });
        let diff = &self.diffs[file];
        DiffView::new(("git_diff", file), diff).split(self.split_diff).show(ui);
    }

    fn palette(&mut self, ctx: &Context) {
        let mut checkout = None;
        let mut action = None;
        Command::new("git_palette").show_dialog(ctx, &mut self.palette_open, |cmd| {
            cmd.group("Repository", |cmd| {
                for (name, key) in [("Fetch", "⌘F"), ("Pull", "⌘P"), ("Push", "⇧⌘P"), ("Commit…", "⌘↵")] {
                    if cmd.item(name, Some(key)) {
                        action = Some(name);
                    }
                }
            });
            cmd.group("Branches", |cmd| {
                for (i, b) in BRANCHES.iter().enumerate() {
                    if cmd.item_with_keywords(&format!("Checkout {b}"), "switch branch", None) {
                        checkout = Some(i);
                    }
                }
            });
            cmd.group("View", |cmd| {
                if cmd.item("Toggle dark mode", None) {
                    action = Some("theme");
                }
                if cmd.item("Toggle split diff", None) {
                    action = Some("split");
                }
            });
        });
        if let Some(i) = checkout {
            self.branch = Some(i);
            toast(ctx, format!("Switched to {}", BRANCHES[i]), None);
        }
        match action {
            Some("theme") => self.dark = !self.dark,
            Some("split") => self.split_diff = !self.split_diff,
            Some(a) => toast(ctx, format!("{a} finished"), None),
            None => {}
        }
    }
}

fn file_menu(row: &egui::Response, path: &str) {
    context_menu(row, 200.0, |ui| {
        if menu_item(ui, "Stage", None).clicked() {
            toast(ui.ctx(), "Staged", Some(path));
        }
        menu_item(ui, "Open in editor", None);
        menu_separator(ui);
        if menu_item(ui, "Discard changes", None).clicked() {
            toast_error(ui.ctx(), "Changes discarded", Some(path));
        }
    });
}
