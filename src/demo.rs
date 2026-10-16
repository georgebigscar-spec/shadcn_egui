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
    [("src/command.rs", "A"), ("src/resizable.rs", "A"), ("src/overlay.rs", "M"), ("README.md", "M")];

/// A small git client layout that uses Resizable, Combobox, Command and
/// ContextMenu together. Used by the `git_client` example and the screenshot test.
pub struct GitDemo {
    pub dark: bool,
    /// Widths of the changes | history | details panels.
    pub sizes: [f32; 3],
    pub palette_open: bool,
    pub branch: Option<usize>,
    pub commit: usize,
    file: Option<&'static str>,
    commits: Vec<Vec<String>>,
}

impl Default for GitDemo {
    fn default() -> Self {
        let subjects = [
            "Add Command palette and Combobox",
            "Resizable panel groups",
            "Context menu on tree and table rows",
            "DataTable: custom cells and cached filter/sort",
            "Tree: row content and multi-select",
            "Fix scrolling in Tree with max_height",
            "Initial commit",
        ];
        let commits = subjects
            .iter()
            .enumerate()
            .map(|(i, s)| {
                vec![
                    s.to_string(),
                    ["George", "Ada", "Linus"][i % 3].to_string(),
                    format!("{:07x}", (i + 1) * 2_654_435_761usize % 0xfff_ffff),
                ]
            })
            .collect();
        Self {
            dark: false,
            sizes: [0.24, 0.46, 0.30],
            palette_open: false,
            branch: Some(0),
            commit: 0,
            file: Some("src/command.rs"),
            commits,
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

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(t.background).inner_margin(16))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    h3(ui, "shadcn_egui");
                    ui.add(Combobox::new(&mut self.branch, &BRANCHES)
                        .placeholder("Select branch")
                        .search_placeholder("Search branch...")
                        .empty("No branch found.")
                        .width(240.0));
                    for action in ["Fetch", "Pull", "Push"] {
                        if ui.add(Button::outline(action).size(ButtonSize::Sm)).clicked() {
                            toast(ui.ctx(), format!("{action} finished"), None);
                        }
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
                                0 => self.changes(ui),
                                1 => self.history(ui),
                                _ => self.details(ui),
                            });
                        self.sizes = sizes;
                    });
            });

        self.palette(ctx);
        Toaster::show(ctx);
    }

    fn changes(&mut self, ui: &mut Ui) {
        let t = Theme::get(ui.ctx());
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            ui.label(RichText::new("Changes").strong());
        });
        ui.add_space(4.0);
        egui::Frame::new().inner_margin(egui::Margin::symmetric(8, 0)).show(ui, |ui| {
            Tree::new("git_changes").default_open_depth(2).show(ui, &mut self.file, |tree| {
                tree.folder("src", "src", |tree| {
                    for (path, status) in CHANGES.iter().filter(|(p, _)| p.starts_with("src/")) {
                        let row = tree.leaf_ui(*path, path, |ui| {
                            ui.label(&path[4..]);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(*status).monospace().color(t.muted_foreground));
                            });
                        });
                        file_menu(&row, path);
                    }
                });
                let row = tree.leaf("README.md", "README.md");
                file_menu(&row, "README.md");
            });
        });
    }

    fn history(&mut self, ui: &mut Ui) {
        let height = ui.available_height() - 40.0;
        egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
            let commits = &self.commits;
            let r = DataTable::new(
                "git_history",
                vec![Column::new("Subject").weight(3.0), Column::new("Author"), Column::new("Hash").right()],
            )
            .row_height(32.0)
            .max_height(height - 40.0)
            .row_context_menu(|ui, row| {
                menu_label(ui, &commits[row][2]);
                menu_separator(ui);
                for action in ["Checkout", "Cherry-pick", "Revert"] {
                    if menu_item(ui, action, None).clicked() {
                        toast(ui.ctx(), format!("{action} {}", commits[row][2]), None);
                    }
                }
                menu_separator(ui);
                menu_item(ui, "Copy hash", Some("⌘C"));
            })
            .show(ui, commits);
            if let Some(row) = r.clicked {
                self.commit = row;
            }
        });
    }

    fn details(&mut self, ui: &mut Ui) {
        let t = Theme::get(ui.ctx());
        let c = &self.commits[self.commit];
        egui::Frame::new().inner_margin(egui::Margin::same(16)).show(ui, |ui| {
            ui.label(RichText::new(&c[0]).size(16.0).color(t.foreground));
            muted(ui, format!("{} · {}", c[1], c[2]));
            ui.add_space(8.0);
            separator(ui);
            ui.add_space(4.0);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            let green = egui::Color32::from_rgb(22, 163, 74);
            for (sign, line) in [
                (' ', "pub use button::{Button, ButtonSize};"),
                ('+', "pub use command::{Combobox, Command};"),
                (' ', "pub use data_table::{Column, DataTable};"),
                ('-', "pub use layout::{tabs, Card};"),
                ('+', "pub use layout::{tabs, Card, Table};"),
                ('+', "pub use resizable::Resizable;"),
            ] {
                let color = match sign {
                    '+' => green,
                    '-' => t.destructive,
                    _ => t.muted_foreground,
                };
                ui.label(RichText::new(format!("{sign} {line}")).monospace().size(12.0).color(color));
            }
        });
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
            });
        });
        if let Some(i) = checkout {
            self.branch = Some(i);
            toast(ctx, format!("Switched to {}", BRANCHES[i]), None);
        }
        match action {
            Some("theme") => self.dark = !self.dark,
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
