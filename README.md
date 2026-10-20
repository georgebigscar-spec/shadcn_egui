# shadcn_egui

[shadcn/ui](https://ui.shadcn.com)-style components for **egui 0.36**. Colors, radii, sizes and states (hover, focus ring, disabled) follow the shadcn "neutral" palette, with light and dark themes.

![light](tests/snapshots/gallery_light.png)

![dark](tests/snapshots/gallery_dark.png)

![git client](tests/snapshots/git_client.png)

## Running

```sh
cargo run --example gallery     # demo gallery on eframe
cargo run --example git_client  # a git client layout: sidebar, commit graph, diff, tabs, Ctrl+K palette
cargo run --example settings    # a settings page: Field, Item, InputOtp, ScrollArea, typography
UPDATE_SNAPSHOTS=1 cargo test   # re-render the screenshots in tests/snapshots/
```

The screenshot tests use `egui_kittest` with wgpu. On a machine without a GPU you need a software Vulkan driver (on Ubuntu: `apt install mesa-vulkan-drivers`).

## Usage

```toml
[dependencies]
egui = "0.36"
shadcn_egui = { git = "https://github.com/georgebigscar-spec/shadcn_egui" }
```

```rust
use shadcn_egui::*;

// once, for example in App::new or before the first frame
Theme::light().install(ctx);   // or Theme::dark()

Card::new()
    .title("Create an account")
    .description("Enter your email below")
    .width(360.0)
    .show(ui, |ui| {
        label(ui, "Email");
        ui.add(Input::new(&mut email).placeholder("m@example.com"));
        ui.add(Checkbox::new(&mut terms).label("Accept terms"));
        if ui.add(Button::new("Create account").full_width()).clicked() {
            toast(ui.ctx(), "Account created", None);
        }
    });

Toaster::show(ctx); // once per frame, if you use toasts
```

`Theme::install` also restyles egui's built-in widgets (windows, scroll bars, selection, TextEdit), so you can mix them freely with the components from this crate.

## Components

| shadcn/ui | shadcn_egui | Notes |
|---|---|---|
| Button | `Button::new / secondary / destructive / outline / ghost / link`, `.size(ButtonSize::Sm/Lg/Icon)`, `.full_width()` | hover, pressed, focus ring, disabled via `ui.add_enabled` |
| Badge | `Badge::new(..).secondary() / destructive() / outline()` | |
| Input | `Input::new(&mut s).placeholder(..).password().invalid(bool).width(..)` | red border and ring when `invalid` |
| Textarea | `Textarea::new(&mut s).rows(n)` | |
| Label | `label(ui, ..)`, `muted(ui, ..)`, `h1/h2/h3` | |
| Typography | `h4`, `p`, `lead`, `large`, `small`, `blockquote`, `list(ui, &[..])`, `inline_code` | `p`, `lead`, `blockquote` and list items wrap |
| Field | `Field::new("Email").description(..).error(Option<..>).required().show(ui, \|ui\| ui.add(Input::new(..)))`, `.horizontal()` for switches; `FieldSet::new("Legend").description(..).show(ui, ..)`, `field_separator(ui, Some("Or"))` | an error turns the label red and replaces the description |
| Input OTP | `InputOtp::new(&mut code, 6).groups(&[3, 3]).alphanumeric().invalid(bool)` | typing, Backspace (Ctrl+Backspace clears) and paste; `changed()` on every edit |
| Item | `Item::new("origin").description(url).icon("🔗")` or `.avatar("PD")`, `.outline() / .muted() / .small() / .clickable()`, `.show(ui, \|ui\| actions)` | actions on the right, returns the row's response |
| Scroll Area | `ScrollArea::vertical().max_height(h).always_visible().show(ui, ..)` | thin floating thumb in the border color, shown on hover; `.with(\|s\| ..)` for other `egui::ScrollArea` settings |
| Checkbox | `Checkbox::new(&mut b).label(..)` | |
| Switch | `Switch::new(&mut b).label(..)` | animated |
| Radio Group | `radio_group(ui, &mut v, &[(val, "Label"), ..])`, `RadioItem` | any `PartialEq` value |
| Slider | `Slider::new(&mut f, 0.0..=100.0).step(1.0)` | dragging and arrow keys |
| Progress | `Progress::new(0.4)` | animates value changes |
| Toggle | `Toggle::new(&mut b, "B").outline()` | |
| Toggle Group | `toggle_group(ui, &mut idx, &["Left", "Center", "Right"])` | single selection |
| Select | `Select::new(&mut Option<usize>, &OPTIONS).placeholder(..)` | popup list with a check mark |
| Dropdown Menu | `dropdown_menu(&trigger, width, \|ui\| { menu_label, menu_item, menu_checkbox, menu_separator })` | shortcuts on the right |
| Combobox | `Combobox::new(&mut Option<usize>, &branches).placeholder(..).search_placeholder(..).empty(..)` | Select with a search field; options can be `&str` or `String`; fuzzy filter, arrow keys, Enter |
| Context Menu | `context_menu(&response, width, \|ui\| { menu_item, .. })` | right click on any clickable response (tree rows, buttons); `DataTable::row_context_menu(\|ui, row\| ..)` for table rows |
| Command | `Command::new(id).show(ui, \|cmd\| { cmd.group("Branch", \|cmd\| if cmd.item("Checkout…", Some("⌘B")) { .. }) })` | search field over a list: fuzzy filter (`fuzzy_match`), groups hide when empty, arrow keys and Enter, `item_with_keywords`, "No results found." |
| Command Dialog | `Command::new(id).show_dialog(ctx, &mut open, \|cmd\| ..)` | Ctrl+K palette in a modal; closes on pick, Escape or click outside. Bind the key yourself with `consume_shortcut` (see `demo::GitDemo`) |
| Resizable | `Resizable::horizontal(id, &mut [0.25, 0.45, 0.3]).min_size(160.0).with_handle().show(ui, \|ui, panel\| ..)` | also `vertical`; the fractions are updated while dragging, so keep them in your app state |
| Menubar | `menubar(ui, id, \|mb\| { mb.menu("File", \|ui\| { menu_item(ui, "Open", Some("⌘O")); }); })` | hovering another trigger while one menu is open switches to it |
| Hover Card | `hover_card(&response, 300.0, \|ui\| ..)` | popover after a hover delay, e.g. a commit card on its hash |
| Sheet | `Sheet::new(id, "Title").description(..).side(SheetSide::Right).size(384.0).show(ctx, &mut open, \|ui\| ..)` | slides in from any edge over a backdrop; Escape, backdrop click or ✕ closes it |
| Calendar | `Calendar::new(id).show(ui, &mut Option<Date>)`, `.show_range(ui, &mut DateRange)` | `.monday_first()`, `.min_date(..)` / `.max_date(..)`, `.today(..)`; ships its own small `Date` type (`Date::new`, `today`, `add_days`, `weekday`, `iso`, `long`), so no date crate is needed |
| Date Picker | `DatePicker::new(id, &mut Option<Date>)`, `DateRangePicker::new(id, &mut DateRange)` | Calendar in a popover; `.calendar(\|c\| c.max_date(Date::today()))` configures it; the range picker closes once both ends are picked |
| Pagination | `pagination(ui, &mut page, page_count)` | `‹ Previous 1 … 4 5 6 … 20 Next ›`, 0-based `page` |
| Popover | `popover(&trigger, width, \|ui\| ..)` | closes on click outside |
| Tooltip | `ui.add(..).tooltip("text")` (`TooltipExt` trait) | dark bubble, as in shadcn |
| Dialog | `Dialog::new(id, "Title").description(..).show(ctx, \|ui\| ..)` + `footer(ui, ..)` | backdrop, ✕ button, Esc, click outside |
| Alert Dialog | `alert_dialog(ctx, id, title, text, "Continue") -> Option<bool>` | |
| Sonner / Toast | `toast(ctx, ..)`, `toast_error(ctx, ..)`, `Toast::new(..).description(..).success() / info() / warning() / error() / loading().action("Undo").show(ctx)`, `Toaster::show(ctx)` | stacked bottom-right, dismissed after 4 s (`.duration(s)`); hovering pauses the timers; loading toasts stay until `Toast::new(..).success().update(ctx, id)`; `Toaster::show` returns the `ToastId` whose action was clicked |
| Card | `Card::new().title(..).description(..).width(..).show(ui, ..)` | |
| Alert | `Alert::new(..).icon("ℹ").description(..).destructive().show(ui)` | |
| Tabs | `tabs(ui, &mut idx, &["Account", "Password"])` | you draw the content for `idx` |
| Accordion | `Accordion::new(id).item(ui, "Title", \|ui\| ..)` | one open section at a time, animated |
| Table | `Table::new(&headers).widths(..).right_align(&[3]).show(ui, &rows)` | row hover, returns the clicked row and the filtered, sorted row order (`visible`) |
| Data Table | `DataTable::new(id, vec![Column::new("Email").sortable(), Column::new("Amount").sortable().right()]).filter("Filter...").selection(&mut set).page_size(10).show(ui, &rows)` | sorting (numbers by value), filter across all cells, checkbox column with select-all, Previous/Next pages, `.max_height(px)` scrolls the rows under a fixed header and only lays out the rows in view, `.row_height(px)`, `.cell_ui(\|ui, row, col\| ..)` for your own cells (graphs, badges); filter and sort results are cached (`.version(n)` invalidates); returns the clicked row |
| Tree | `Tree::new(id).default_open_depth(1).show(ui, &mut selected, \|tree\| { tree.folder(v, "src", \|tree\| ..); tree.leaf(v, "main.rs"); })` | not in shadcn/ui itself, styled like its sidebar file tree; any `PartialEq` value, folder/file icons, guide lines, arrow keys, `.max_height(px)` to scroll; `leaf_ui` / `folder_ui` for your own row content; pass a `HashSet<T>` or `Vec<T>` for multiple selection (Ctrl/Shift+click, Ctrl+A); `.filter(query)` shows only fuzzy matches and the folders leading to them; `tree.leaf_status(v, "main.rs", "M")` adds a colored git status letter (`git_status_color`); rows out of view are not drawn |
| Tab Bar | `TabBar::new(id).show(ui, &mut open_files, &mut active, \|f\| f.name.clone())` | not in shadcn/ui: editor-style closable tabs (✕, middle click, Ctrl+W); returns the closed item; scrolls the active tab into view |
| Sidebar | `Sidebar::new(id).show(ui, &mut view, \|sb\| { sb.item(View::Changes, "Changes", Some("4")); sb.group("Branches", \|sb\| ..) })` | collapsible groups, selected item, count badges; `Sidebar::fill(&theme)` for the panel background |
| Spinner | `ui.add(Spinner::new().size(16.0))` | |
| Empty | `Empty::new("No changes").icon("✓").description(..).show(ui, \|ui\| buttons)` | |
| Diff (extra) | `DiffView::new(id, &git_diff_output).split(side_by_side).show(ui)` | parses unified diffs: file and hunk headers, old/new line numbers, green/red lines; side-by-side pairs deletions with additions and highlights the changed words; only visible rows are drawn. `.search(query).current_match(i)` highlights hits and scrolls to the current one (`response.matches` counts them). `.selection(&mut BTreeSet<usize>)` makes lines clickable (Shift+click for a range, a hunk header for the whole hunk), and `partial_patch(diff, &selected, reverse)` builds the patch for `git apply --cached [--reverse]` to stage or unstage just those lines |
| Commit graph (extra) | `CommitGraph::new(commits.iter().map(\|c\| (id, parents)))`, then `graph.paint(ui, row)` in `DataTable::cell_ui` | lanes, merge commits, branches bending back into their parent's lane; `RefBadge::head / branch / remote / tag(name).color(graph.lane_color(row))` labels a commit with its refs |
| Merge conflicts (extra) | `MergeView::new(id, &file_text).scroll_to_conflict(i).show(ui, &mut Vec<Resolution>)`, then `resolve_conflicts(&file_text, &resolutions)` | parses git's `<<<<<<<` / `=======` / `>>>>>>>` blocks (and `|||||||` diff3 bases); per block: current, incoming (and base) with Accept current / incoming / both / base, Undo once resolved; long unchanged runs fold into a clickable row; `count_conflicts(text)`; the response says how many are resolved |
| Button Group | `button_group(ui, &["Fetch", "Pull", "Push"]) -> Vec<Response>` | joined outline buttons; attach a `dropdown_menu` to any response |
| Input Group | `InputGroup::new(&mut s).placeholder(..).search_icon().clearable().suffix("⌘F")` | addons inside the input's border |
| Breadcrumb | `breadcrumb(ui, &["repo", "src", "main.rs"]) -> Option<usize>` | the last segment is the current page; returns a clicked segment |
| Collapsible | `Collapsible::new(id, "Staged changes").trailing("3").default_open(true).show(ui, \|ui\| ..)` | animated; any number can be open, unlike Accordion |
| Separator | `separator(ui)` | horizontal or vertical depending on layout |
| Avatar | `Avatar::new("CN").image(src).size(40.0)` | initials until the image loads |
| Skeleton | `Skeleton::new([w, h])`, `Skeleton::circle(d)` | pulsing |
| Kbd | `kbd(ui, "⌘")` | |

## Limitations

- egui's default font has no bold weight, so headings are not bold. For an exact look, load Inter or Geist with `ctx.set_fonts` and give headings their own font family.
- Not implemented yet: Navigation Menu, Carousel, Chart, Drawer (use `Sheet` from the bottom). For charts there is the `egui_plot` crate.
- Colors are hex approximations of the shadcn v4 oklch variables; every field of `Theme` can be changed.

## License

MIT, see [LICENSE](LICENSE).
