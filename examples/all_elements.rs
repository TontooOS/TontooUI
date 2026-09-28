//! All-elements gallery: every TontooUI element on one scrollable page.
//!
//! A single `ScrollView` holds a flat `VStack` where each element follows
//! its caption, so the whole catalogue is reachable by scrolling. Modal
//! elements (`BasicAlert`, `ActionAlert`, `ConfirmationDialog`,
//! `IconAlert`, `BasicSheet`, the `ColorPicker` popup and the
//! `ContextMenu`) live as overlays and open through demo buttons;
//! right-clicking an editable table cell opens the table context menu.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tontooui::elements::{
    ActionAlert, AlertAction, AlertButton, Align, Animated, AppImage, Background,
    BarItem, BarMenu, BasicAlert, BasicGroupBox, BasicLabel, BasicLink, BasicList,
    BasicOutlineGroup, BasicSheet, BasicTable, BasicText, BasicTextField, BasicToolbar,
    Button, ButtonStyle, CapacityGauge, Capsule, Circle, CircularGauge, ColorPicker,
    ConfirmationDialog, ContentUnavailable, ContextMenu, CustomContentUnavailable,
    CustomShape, DatePicker, DisclosureGroup, FontPreview, Form, FormRow, FormSection,
    FormattedText, Frame, Gauge, GestureArea, GlassContainer, GlassType, GradientPaint,
    HStack, HorizontalDivider, IconAlert, IconLabel, ImageFit, ImageLabel, ImageOverlay,
    InlinePicker, LabelStyle, LabeledText, LargeTextEditor, LargeTextField, LinearGauge,
    LinearProgress, LinkStyle, LinkWithImage, Material, MaterialKind, Menu, MenuButton,
    MenuItem, MenuPicker, NestedMenu, OutlineNode, Padding, Rectangle, RoundedRectangle,
    ScrollView, Scrollbar, SearchEmpty, SearchField, SecureField, SegmentedPicker,
    SFSymbolImage, SheetSize, Sidebar, SidebarItem, Slider, Span, Spinner, Stepper,
    StyledGroupBox, StyledLabel, StyledLink, TableColumn, TableHit, TextEditor,
    TextStyle, Titlebar, Toggle, ToggleStyle, ToolbarItem, ToolbarPlacement,
    TrafficAction, UrlImage, VerticalDivider, View, VStack, ZStack, ALL_SYSTEM_COLORS,
    Spacer, SystemColor,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, CursorKind, Key, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

fn accent_blue() -> Color {
    Color::from_rgb8(0x0b, 0x5c, 0xe6)
}

fn accent_purple() -> Color {
    Color::from_rgb8(0xaf, 0x52, 0xde)
}

fn accent_green() -> Color {
    Color::from_rgb8(0x34, 0xc7, 0x59)
}

struct AllElements {
    bar: Titlebar,
    scroll: ScrollView,
    alert_ok: BasicAlert,
    alert_both: BasicAlert,
    alert_action: ActionAlert,
    alert_dialog: ConfirmationDialog,
    alert_icon: IconAlert,
    sheet: BasicSheet<VStack>,
    context: ContextMenu,
    picker: ColorPicker,
    show_ok: Rc<RefCell<bool>>,
    show_both: Rc<RefCell<bool>>,
    show_action: Rc<RefCell<bool>>,
    show_dialog: Rc<RefCell<bool>>,
    show_icon: Rc<RefCell<bool>>,
    open_sheet: Rc<RefCell<Option<SheetSize>>>,
    dismiss_sheet: Rc<RefCell<bool>>,
    show_picker: Rc<RefCell<bool>>,
    table_pending: Rc<Cell<Option<(usize, usize)>>>,
    table_edit: Rc<Cell<bool>>,
    sidebar_selected: Rc<Cell<usize>>,
    form_status: Rc<RefCell<String>>,
    text_cursor: Cell<bool>,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl AllElements {
    fn new() -> Self {
        let show_ok: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let show_both: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let show_action: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let show_dialog: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let show_icon: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let open_sheet: Rc<RefCell<Option<SheetSize>>> = Rc::new(RefCell::new(None));
        let dismiss_sheet: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let show_picker: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let table_pending: Rc<Cell<Option<(usize, usize)>>> = Rc::new(Cell::new(None));
        let table_edit: Rc<Cell<bool>> = Rc::new(Cell::new(false));
        let sidebar_selected: Rc<Cell<usize>> = Rc::new(Cell::new(0));
        let form_status: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

        let mut page = VStack::new().spacing(10.0).align(Align::Leading);

        // Layout stacks and modifiers.
        page = page
            .child(BasicText::new("01 - HStack, Spacer and Button"))
            .child(
                HStack::new()
                    .spacing(12.0)
                    .child(Button::new("Left"))
                    .child(Spacer::new())
                    .child(Button::new("Right")),
            )
            .child(BasicText::new("02 - ZStack"))
            .child(
                ZStack::new()
                    .child(BasicText::new("Back layer"))
                    .child(BasicText::new("Front layer")),
            )
            .child(BasicText::new("03 - Padding"))
            .child(Padding::all(BasicText::new("Padded text"), 12.0))
            .child(BasicText::new("04 - Background"))
            .child(
                Background::new(
                    BasicText::new("Text on a rounded background"),
                    Color::from_rgb8(0x80, 0x80, 0x80),
                )
                .radius(12.0),
            )
            .child(BasicText::new("05 - Frame"))
            .child(Frame::new(BasicText::new("Fixed 260x40 box"), 260.0, 40.0));

        // Text.
        page = page
            .child(BasicText::new("06 - BasicText styles"))
            .child(BasicText::new("Title text").style(TextStyle::Title))
            .child(BasicText::new("Headline text").style(TextStyle::Headline))
            .child(BasicText::new("Gradient text").foreground_gradient(vec![
                Color::from_rgb8(0xff, 0x3b, 0x30),
                Color::from_rgb8(0xff, 0xcc, 0x00),
                Color::from_rgb8(0x34, 0xc7, 0x59),
                Color::from_rgb8(0x00, 0x7a, 0xff),
                Color::from_rgb8(0xaf, 0x52, 0xde),
            ]))
            .child(BasicText::new("07 - FormattedText and Span"))
            .child(FormattedText::markdown("**Bold** and *italic* together"))
            .child(FormattedText::spans(vec![
                Span::new("Underlined").underline(),
                Span::new(" and bold").bold(),
            ]))
            .child(BasicText::new("08 - LabeledText"))
            .child(LabeledText::new("Starred", "star").style(TextStyle::Title2))
            .child(BasicText::new("09 - FontPreview"))
            .child(FontPreview::new("SF Pro"));

        // Text fields.
        page = page
            .child(BasicText::new("10 - BasicTextField"))
            .child(BasicTextField::new("Enter text here"))
            .child(BasicText::new("11 - LargeTextField"))
            .child(LargeTextField::new("Large field"))
            .child(BasicText::new("12 - SecureField"))
            .child(SecureField::new("Password"))
            .child(BasicText::new("13 - SearchField"))
            .child(SearchField::new("Search items"))
            .child(BasicText::new("14 - TextEditor"))
            .child(TextEditor::new("Write something"))
            .child(BasicText::new("15 - LargeTextEditor"))
            .child(LargeTextEditor::new("Start typing here"));

        // Buttons and toggles.
        page = page
            .child(BasicText::new("16 - Button styles"))
            .child(Button::new("Automatic"))
            .child(Button::new("Bordered").style(ButtonStyle::Bordered))
            .child(
                Button::new("BorderedProminent").style(ButtonStyle::BorderedProminent),
            )
            .child(Button::new("BorderedTinted").style(ButtonStyle::BorderedTinted))
            .child(Button::new("Plain").style(ButtonStyle::Plain))
            .child(BasicText::new("17 - Toggle styles"))
            .child(Toggle::new("Wi-Fi").on(true))
            .child(
                Toggle::new("Airplane Mode")
                    .style(ToggleStyle::Checkbox)
                    .on(true),
            )
            .child(Toggle::new("Button toggle").style(ToggleStyle::Button));

        // Sliders, steppers, progress and gauges.
        page = page
            .child(BasicText::new("18 - Slider"))
            .child(Slider::new(0.6, 0.0, 1.0).title("Volume"))
            .child(BasicText::new("19 - Stepper"))
            .child(Stepper::new(5.0, 0.0, 10.0))
            .child(BasicText::new("20 - LinearProgress"))
            .child(LinearProgress::new().title("Downloading"))
            .child(BasicText::new("21 - Spinner"))
            .child(Spinner::new().text("Syncing"))
            .child(BasicText::new("22 - Gauges"))
            .child(Gauge::new(0.7, 0.0, 1.0).title("Battery"))
            .child(LinearGauge::new(60.0, 0.0, 100.0))
            .child(CircularGauge::new(70.0, 0.0, 100.0))
            .child(CapacityGauge::new(65.0, 0.0, 100.0));

        // Pickers.
        page = page
            .child(BasicText::new("23 - SegmentedPicker"))
            .child(SegmentedPicker::from_slice("Size", &["Small", "Medium", "Large"]))
            .child(BasicText::new("24 - InlinePicker"))
            .child(InlinePicker::from_slice("Choice", &["A", "B", "C"]))
            .child(BasicText::new("25 - MenuPicker"))
            .child(MenuPicker::from_slice("Color", &["Red", "Green", "Blue"]))
            .child(BasicText::new("26 - MenuButton"))
            .child(MenuButton::from_slice("Actions", &["Edit", "Delete"]))
            .child(BasicText::new("27 - DatePicker"))
            .child(DatePicker::new().selected(2026, 9, 28));

        // Menus.
        page = page
            .child(BasicText::new("28 - Menu"))
            .child(Menu::from_slice("Options", &["One", "Two", "Three"]))
            .child(BasicText::new("29 - NestedMenu and MenuItem"))
            .child(NestedMenu::new(
                "File",
                vec![
                    MenuItem::action("New"),
                    MenuItem::submenu(
                        "Open Recent",
                        vec![MenuItem::action("notes.txt")],
                    ),
                    MenuItem::divider(),
                    MenuItem::action("Quit"),
                ],
            ))
            .child(BasicText::new("30 - BarMenu and BarItem"))
            .child(
                BarMenu::new(vec![
                    BarItem::action("File").shortcut("Ctrl O").build(),
                    BarItem::divider(),
                    BarItem::action("Edit").icon("pencil").build(),
                ])
                .on_action(|path| println!("bar menu: {path:?}")),
            )
            .child(BasicText::new(
                "31 - ContextMenu (right-click an editable table cell)",
            ));

        // Lists and outlines.
        page = page
            .child(BasicText::new("32 - BasicList"))
            .child(BasicList::from_slice(&["Mercury", "Venus", "Earth"]))
            .child(BasicText::new("33 - DisclosureGroup"))
            .child(DisclosureGroup::from_slice(
                "Outer planets",
                &["Jupiter", "Saturn"],
            ))
            .child(BasicText::new("34 - BasicOutlineGroup and OutlineNode"))
            .child(BasicOutlineGroup::new(vec![
                OutlineNode::folder("Documents").expanded(true).children(vec![
                    OutlineNode::file("Notes.txt"),
                    OutlineNode::file("Sketch.png"),
                ]),
                OutlineNode::folder("Downloads")
                    .child(OutlineNode::file("Archive.zip")),
            ]));

        // Table.
        let columns = vec![
            TableColumn::new("Name").weight(1.4).editable(true),
            TableColumn::new("Age").weight(0.6).min_width(60.0),
            TableColumn::new("City")
                .weight(1.0)
                .sort_by(|a, b| a.len().cmp(&b.len())),
            TableColumn::new("Email")
                .weight(1.6)
                .min_width(160.0)
                .editable(true),
        ];
        let rows: Vec<Vec<String>> = [
            ["Ada", "36", "Berlin", "ada@tontoo.os"],
            ["Bob", "34", "Munich", "bob@tontoo.os"],
            ["Cleo", "29", "Hamburg", "cleo@tontoo.os"],
            ["Dan", "41", "Cologne", "dan@tontoo.os"],
            ["Eve", "25", "Ulm", "eve@tontoo.os"],
        ]
        .iter()
        .map(|row| row.iter().map(|cell| cell.to_string()).collect())
        .collect();
        page = page
            .child(BasicText::new(
                "35 - BasicTable and TableColumn (click headers to sort)",
            ))
            .child(BasicTable::new(columns, rows).selectable(true));

        // Form.
        let saved = form_status.clone();
        let cancelled = form_status.clone();
        page = page
            .child(BasicText::new("36 - Form, FormSection and FormRow"))
            .child(
                Form::new()
                    .section(
                        FormSection::titled("Connection")
                            .row(FormRow::text("Username", "octo"))
                            .row(FormRow::text("Host", "tontoo.os")),
                    )
                    .section(
                        FormSection::titled("Authentication")
                            .row(FormRow::secure("Password", ""))
                            .row(FormRow::toggle("Use SSH Key", false)),
                    )
                    .section(
                        FormSection::titled("Options")
                            .row(FormRow::picker(
                                "Protocol",
                                vec!["FTP".into(), "SFTP".into(), "WebDAV".into()],
                                1,
                            ))
                            .footnote("Pickers open a glass panel."),
                    )
                    .section(
                        FormSection::new().row(FormRow::buttons(vec![
                            Button::new("Cancel").style(ButtonStyle::Bordered).on_press(
                                move || {
                                    *cancelled.borrow_mut() = "Cancelled".to_string();
                                },
                            ),
                            Button::new("Save")
                                .style(ButtonStyle::BorderedProminent)
                                .on_press(move || {
                                    *saved.borrow_mut() = "Saved".to_string();
                                }),
                        ])),
                    ),
            );

        // Toolbars.
        page = page
            .child(BasicText::new("37 - BasicToolbar, ToolbarItem"))
            .child(
                BasicToolbar::from_items(vec![
                    ToolbarItem::icon("chevron.left"),
                    ToolbarItem::divider(),
                    ToolbarItem::icon("chevron.right"),
                ])
                .placement(ToolbarPlacement::Leading)
                .on_action(|index| println!("toolbar action {index}")),
            )
            .child(
                BasicToolbar::from_icons(vec![
                    "heart".to_string(),
                    "star".to_string(),
                    "checkmark".to_string(),
                ])
                .placement(ToolbarPlacement::Center),
            );

        // Sidebar (inline preview; traffic lights stay decorative here).
        let sidebar_select = sidebar_selected.clone();
        page = page
            .child(BasicText::new("38 - Sidebar and SidebarItem"))
            .child(
                Sidebar::new(vec![
                    SidebarItem::new("General", "gear"),
                    SidebarItem::new("Security", "lock.fill"),
                    SidebarItem::new("Storage", "internaldrive"),
                ])
                .page(
                    VStack::new()
                        .spacing(8.0)
                        .child(BasicText::new("General settings"))
                        .child(BasicText::new("System appearance and behavior.")),
                )
                .page(
                    VStack::new()
                        .spacing(8.0)
                        .child(BasicText::new("Security settings"))
                        .child(BasicText::new("Passwords and encryption.")),
                )
                .page(
                    VStack::new()
                        .spacing(8.0)
                        .child(BasicText::new("Storage settings"))
                        .child(BasicText::new("Disks and usage.")),
                )
                .on_select(move |index| sidebar_select.set(index)),
            );

        // Alert demo buttons (the overlays live outside the scroll view).
        let alert_button = |label: &str, flag: Rc<RefCell<bool>>| {
            Button::new(label)
                .style(ButtonStyle::Bordered)
                .on_press(move || {
                    *flag.borrow_mut() = true;
                })
        };
        page = page
            .child(BasicText::new(
                "39 - Alerts (BasicAlert, ActionAlert, ConfirmationDialog, IconAlert)",
            ))
            .child(alert_button("Show OK Alert", show_ok.clone()))
            .child(alert_button("Show OK / Cancel Alert", show_both.clone()))
            .child(alert_button("Show Action Alert", show_action.clone()))
            .child(alert_button("Show Confirmation Dialog", show_dialog.clone()))
            .child(alert_button("Show Icon Alert", show_icon.clone()));

        // Sheet demo buttons.
        let sheet_button = |label: &str, size: SheetSize, flag: Rc<RefCell<Option<SheetSize>>>| {
            Button::new(label)
                .style(ButtonStyle::BorderedProminent)
                .on_press(move || {
                    *flag.borrow_mut() = Some(size);
                })
        };
        page = page
            .child(BasicText::new("40 - BasicSheet and SheetSize"))
            .child(sheet_button("Small Sheet", SheetSize::Small, open_sheet.clone()))
            .child(sheet_button("Half Sheet", SheetSize::Half, open_sheet.clone()));

        // Colors: inline picker, system colors and gradient paints.
        let linear = GradientPaint::preset_linear();
        let radial = GradientPaint::preset_radial();
        let _angular = GradientPaint::preset_angular();
        page = page
            .child(BasicText::new("41 - ColorPicker"))
            .child(ColorPicker::new())
            .child(
                Button::new("Pick a color (popup)")
                    .style(ButtonStyle::BorderedProminent)
                    .on_press({
                        let flag = show_picker.clone();
                        move || {
                            *flag.borrow_mut() = true;
                        }
                    }),
            )
            .child(BasicText::new("42 - SystemColor and ALL_SYSTEM_COLORS"))
            .child(StyledLabel::status(
                SystemColor::Red.name(),
                SystemColor::Red.color(),
            ))
            .child(BasicText::new(
                "43 - GradientPaint (linear and radial presets)",
            ))
            .child(Rectangle::new(220.0, 80.0).linear_gradient(linear.colors(), 0.0))
            .child(Circle::new(90.0).radial_gradient(radial.colors()));
        for color in ALL_SYSTEM_COLORS {
            page = page.child(StyledLabel::status(color.name(), color.color()));
        }

        // Material and glass.
        page = page
            .child(BasicText::new("44 - Material and MaterialKind"))
            .child(Material::thin(BasicText::new("Thin material")))
            .child(Material::new(
                BasicText::new("Thick material"),
                MaterialKind::Thick,
            ))
            .child(BasicText::new("45 - GlassContainer and GlassType"))
            .child(
                GlassContainer::new()
                    .bounds(0.0, 0.0, 420.0, 110.0)
                    .radius(24.0)
                    .glass_type(GlassType::Frosted)
                    .content(BasicText::new("Frosted liquid glass")),
            );

        // Gestures and animation.
        page = page
            .child(BasicText::new("46 - GestureArea"))
            .child(
                GestureArea::new(Rectangle::new(220.0, 90.0).fill(accent_blue()))
                    .on_tap(|| println!("gesture: tap"))
                    .on_long_press(|| println!("gesture: long press"))
                    .on_drag(|dx, dy| println!("gesture: drag {dx:.1},{dy:.1}"))
                    .on_magnify(|scale| println!("gesture: magnify {scale:.2}")),
            )
            .child(BasicText::new("47 - Animated"))
            .child(Animated::new(
                SFSymbolImage::new("star.fill").size(28.0),
            ));

        // Empty states.
        page = page
            .child(BasicText::new("48 - ContentUnavailable"))
            .child(ContentUnavailable::new(
                "tray",
                "No Data",
                "Import files to begin.",
            ))
            .child(BasicText::new("49 - CustomContentUnavailable"))
            .child(CustomContentUnavailable::new(
                "sparkles",
                "Coming Soon",
                "This gallery keeps evolving.",
                Button::new("Notify Me"),
            ))
            .child(BasicText::new("50 - SearchEmpty"))
            .child(SearchEmpty::new(
                "magnifyingglass",
                "No Results",
                "Try another search.",
            ));

        // Group boxes.
        page = page
            .child(BasicText::new("51 - BasicGroupBox"))
            .child(BasicGroupBox::new("This is content inside a GroupBox."))
            .child(BasicText::new("52 - StyledGroupBox"))
            .child(
                StyledGroupBox::new()
                    .check_row("Notifications", true)
                    .toggle_row("Wi-Fi", true)
                    .symbol_row("wifi", "Network"),
            );

        // Labels.
        page = page
            .child(BasicText::new(
                "53 - BasicLabel, IconLabel, ImageLabel, StyledLabel, LabelStyle",
            ))
            .child(BasicLabel::new("star", "Star"))
            .child(IconLabel::new("heart").icon_color(accent_green()))
            .child(ImageLabel::new(
                "https://picsum.photos/96/96",
                "Custom Title",
            ))
            .child(StyledLabel::status("Downloaded", accent_green()))
            .child(StyledLabel::new("Titled label", LabelStyle::Title));

        // Links.
        page = page
            .child(BasicText::new("54 - BasicLink, StyledLink, LinkWithImage"))
            .child(BasicLink::new("Visit Apple", "https://apple.com"))
            .child(
                StyledLink::new("Border Link", "https://example.com")
                    .style(LinkStyle::Border),
            )
            .child(LinkWithImage::new(
                "https://picsum.photos/320/200",
                "Download App",
                "https://example.com/download",
                320.0,
                200.0,
            ));

        // Images.
        page = page
            .child(BasicText::new(
                "55 - SFSymbolImage, AppImage, UrlImage, ImageOverlay",
            ))
            .child(SFSymbolImage::new("star.fill"))
            .child(
                SFSymbolImage::new("bell.fill")
                    .size(40.0)
                    .color(Color::from_rgb8(0xff, 0x9f, 0x0a)),
            )
            .child(AppImage::new("demo-star", 200.0, 130.0))
            .child(AppImage::new("demo-star", 200.0, 130.0).fit(ImageFit::Fit))
            .child(UrlImage::new("https://picsum.photos/400/260", 200.0, 130.0))
            .child(
                ImageOverlay::resource("demo-star", 220.0, 140.0)
                    .caption("Demo star")
                    .badge("heart.fill"),
            );

        // Shapes.
        page = page
            .child(BasicText::new(
                "56 - Rectangle, Circle, RoundedRectangle, Capsule, CustomShape",
            ))
            .child(Rectangle::new(200.0, 80.0).fill(accent_blue()))
            .child(Circle::new(90.0))
            .child(RoundedRectangle::new(200.0, 80.0, 16.0).fill(accent_purple()))
            .child(Capsule::new(200.0, 70.0))
            .child(
                CustomShape::star(120.0, 120.0)
                    .fill(Color::from_rgb8(0xff, 0x9f, 0x0a)),
            );

        // Dividers, scrollbar and titlebar.
        page = page
            .child(BasicText::new("57 - HorizontalDivider and VerticalDivider"))
            .child(HorizontalDivider::new())
            .child(VerticalDivider::new())
            .child(BasicText::new("58 - Scrollbar and ScrollView"))
            .child(Scrollbar::new())
            .child(BasicText::new(
                "This page itself is the ScrollView demo: it clips and scrolls.",
            ))
            .child(BasicText::new("59 - Titlebar (inline copy)"))
            .child(Titlebar::new("Inline Titlebar"));

        let sheet_content = VStack::new()
            .align(Align::Center)
            .spacing(20.0)
            .child(BasicText::new("This is a sheet!"))
            .child(
                Button::new("Dismiss")
                    .style(ButtonStyle::Bordered)
                    .on_press({
                        let flag = dismiss_sheet.clone();
                        move || {
                            *flag.borrow_mut() = true;
                        }
                    }),
            );

        Self {
            bar: Titlebar::new("All Elements"),
            scroll: ScrollView::new(page),
            alert_ok: BasicAlert::ok("Alert Title", "This is a basic alert message."),
            alert_both: BasicAlert::buttons(
                "Delete Item?",
                "This cannot be undone.",
                vec![AlertButton::cancel("Cancel"), AlertButton::ok("OK")],
            ),
            alert_action: ActionAlert::new(
                "Delete Item?",
                "Are you sure you want to delete this item?",
                AlertButton::cancel("Cancel"),
                AlertButton::ok("Delete").color(Color::from_rgb8(0xff, 0x3b, 0x30)),
            ),
            alert_dialog: ConfirmationDialog::new(
                "Choose Action",
                vec![
                    AlertButton::ok("Option 1"),
                    AlertButton::ok("Option 2"),
                    AlertButton::ok("Option 3"),
                ],
            ),
            alert_icon: IconAlert::new(
                "lock.fill",
                "Authentication Required",
                "Enter an administrator name and password to continue.",
                vec![AlertButton::ok("Use Password...")],
            ),
            sheet: BasicSheet::new(sheet_content),
            context: ContextMenu::basic(
                (0.0, 0.0, 0.0, 0.0),
                Menu::from_slice("Cell", &["Edit Cell"]).on_action({
                    let pending = table_pending.clone();
                    let edit = table_edit.clone();
                    move |index| {
                        if index == 0 && pending.get().is_some() {
                            edit.set(true);
                        }
                    }
                }),
            ),
            picker: ColorPicker::new(),
            show_ok,
            show_both,
            show_action,
            show_dialog,
            show_icon,
            open_sheet,
            dismiss_sheet,
            show_picker,
            table_pending,
            table_edit,
            sidebar_selected,
            form_status,
            text_cursor: Cell::new(false),
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }

    fn stack_mut(&mut self) -> Option<&mut VStack> {
        self.scroll.child_mut::<VStack>()
    }

    fn any_overlay(&self) -> bool {
        self.alert_ok.is_visible()
            || self.alert_both.is_visible()
            || self.alert_action.is_visible()
            || self.alert_dialog.is_visible()
            || self.alert_icon.is_visible()
            || self.sheet.is_visible()
            || self.picker.is_visible()
            || self.context.is_open()
    }

    fn table_mut(&mut self) -> Option<&mut BasicTable> {
        let stack = self.stack_mut()?;
        for index in 0..stack.len() {
            if stack.child_mut::<BasicTable>(index).is_some() {
                return stack.child_mut::<BasicTable>(index);
            }
        }
        None
    }

    /// Theme every direct page child that follows the system theme.
    /// Elements without a theme API keep their defaults.
    fn theme_page(&mut self, accent: Color, dark: bool, mode: ThemeMode, glass: tontooui::theme::GlassAmount, viewport: Viewport, focused: bool) {
        let Some(stack) = self.stack_mut() else {
            return;
        };
        for index in 0..stack.len() {
            if let Some(text) = stack.child_mut::<BasicText>(index) {
                text.set_theme(mode);
                text.set_focused(focused);
                continue;
            }
            if let Some(button) = stack.child_mut::<Button>(index) {
                button.set_theme(accent, dark);
                button.set_focused(focused);
                continue;
            }
            if let Some(toggle) = stack.child_mut::<Toggle>(index) {
                toggle.set_theme(accent, dark);
                toggle.set_focused(focused);
                continue;
            }
            if let Some(slider) = stack.child_mut::<Slider>(index) {
                slider.set_theme(accent, dark, glass);
                slider.set_focused(focused);
                continue;
            }
            if let Some(stepper) = stack.child_mut::<Stepper>(index) {
                stepper.set_theme(accent, dark);
                stepper.set_focused(focused);
                continue;
            }
            if let Some(bar) = stack.child_mut::<LinearProgress>(index) {
                bar.set_theme(accent, dark);
                bar.set_focused(focused);
                bar.set_progress(0.65);
                continue;
            }
            if let Some(gauge) = stack.child_mut::<Gauge>(index) {
                gauge.set_theme(accent, dark);
                gauge.set_focused(focused);
                continue;
            }
            if let Some(gauge) = stack.child_mut::<LinearGauge>(index) {
                gauge.set_theme(accent, dark);
                gauge.set_focused(focused);
                continue;
            }
            if let Some(gauge) = stack.child_mut::<CircularGauge>(index) {
                gauge.set_theme(accent, dark);
                gauge.set_focused(focused);
                continue;
            }
            if let Some(gauge) = stack.child_mut::<CapacityGauge>(index) {
                gauge.set_theme(accent, dark);
                gauge.set_focused(focused);
                continue;
            }
            if let Some(field) = stack.child_mut::<BasicTextField>(index) {
                field.set_theme(accent, dark);
                field.set_focused(focused);
                continue;
            }
            if let Some(field) = stack.child_mut::<LargeTextField>(index) {
                field.set_theme(accent, dark);
                field.set_focused(focused);
                continue;
            }
            if let Some(field) = stack.child_mut::<SecureField>(index) {
                field.set_theme(accent, dark);
                field.set_focused(focused);
                continue;
            }
            if let Some(field) = stack.child_mut::<SearchField>(index) {
                field.set_theme(mode, accent, glass);
                field.set_focused(focused);
                continue;
            }
            if let Some(editor) = stack.child_mut::<TextEditor>(index) {
                editor.set_theme(accent, dark);
                editor.set_focused(focused);
                continue;
            }
            if let Some(editor) = stack.child_mut::<LargeTextEditor>(index) {
                editor.set_theme(accent, dark);
                editor.set_focused(focused);
                continue;
            }
            if let Some(picker) = stack.child_mut::<SegmentedPicker>(index) {
                picker.set_theme(accent, dark);
                picker.set_focused(focused);
                continue;
            }
            if let Some(picker) = stack.child_mut::<InlinePicker>(index) {
                picker.set_theme(accent, dark);
                picker.set_focused(focused);
                continue;
            }
            if let Some(picker) = stack.child_mut::<MenuPicker>(index) {
                picker.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
                picker.set_theme(accent, dark);
                picker.set_glass(mode, glass);
                picker.set_focused(focused);
                continue;
            }
            if let Some(menu) = stack.child_mut::<MenuButton>(index) {
                menu.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
                menu.set_theme(accent, dark);
                menu.set_glass(mode, glass);
                menu.set_focused(focused);
                continue;
            }
            if let Some(menu) = stack.child_mut::<Menu>(index) {
                menu.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
                menu.set_theme(accent, dark);
                menu.set_glass(mode, glass);
                menu.set_focused(focused);
                continue;
            }
            if let Some(menu) = stack.child_mut::<NestedMenu>(index) {
                menu.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
                menu.set_theme(accent, dark);
                menu.set_glass(mode, glass);
                menu.set_focused(focused);
                continue;
            }
            if let Some(picker) = stack.child_mut::<DatePicker>(index) {
                picker.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
                picker.set_theme(accent, dark);
                picker.set_glass(mode, glass);
                picker.set_focused(focused);
                continue;
            }
            if let Some(form) = stack.child_mut::<Form>(index) {
                form.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
                form.set_theme(accent, dark);
                form.set_glass(mode, glass);
                form.set_focused(focused);
                continue;
            }
            if let Some(table) = stack.child_mut::<BasicTable>(index) {
                table.set_theme(accent, dark);
                table.set_focused(focused);
                continue;
            }
            if let Some(toolbar) = stack.child_mut::<BasicToolbar>(index) {
                toolbar.set_theme(mode, glass);
                toolbar.set_focused(focused);
                continue;
            }
            if let Some(sidebar) = stack.child_mut::<Sidebar>(index) {
                sidebar.set_theme(accent, dark);
                sidebar.set_glass(mode, glass);
                sidebar.set_focused(focused);
                continue;
            }
            if let Some(picker) = stack.child_mut::<ColorPicker>(index) {
                picker.set_theme(mode, glass);
                picker.set_focused(focused);
                continue;
            }
            if let Some(glass_box) = stack.child_mut::<GlassContainer>(index) {
                glass_box.set_theme(mode, glass);
                glass_box.set_focused(focused);
                continue;
            }
            if let Some(material) = stack.child_mut::<Material<BasicText>>(index) {
                material.set_theme(dark);
                material.set_focused(focused);
                continue;
            }
            if let Some(group) = stack.child_mut::<BasicGroupBox>(index) {
                group.set_theme(mode);
                group.set_focused(focused);
                continue;
            }
            if let Some(outline) = stack.child_mut::<BasicOutlineGroup>(index) {
                outline.set_theme(accent, dark);
                outline.set_focused(focused);
                continue;
            }
            if let Some(row) = stack.child_mut::<HStack>(index) {
                row.set_focused(focused);
                continue;
            }
            if let Some(row) = stack.child_mut::<ZStack>(index) {
                row.set_focused(focused);
                continue;
            }
            if let Some(padded) = stack.child_mut::<Padding>(index) {
                padded.set_focused(focused);
                continue;
            }
            if let Some(framed) = stack.child_mut::<Frame>(index) {
                framed.set_focused(focused);
                continue;
            }
            if let Some(background) = stack.child_mut::<Background>(index) {
                background.set_focused(focused);
                continue;
            }
        }
    }

    fn forward_text(&mut self, text: &str) {
        let Some(stack) = self.stack_mut() else {
            return;
        };
        for index in 0..stack.len() {
            if let Some(field) = stack.child_mut::<BasicTextField>(index) {
                field.type_text(text);
                continue;
            }
            if let Some(field) = stack.child_mut::<LargeTextField>(index) {
                field.type_text(text);
                continue;
            }
            if let Some(field) = stack.child_mut::<SecureField>(index) {
                field.type_text(text);
                continue;
            }
            if let Some(field) = stack.child_mut::<SearchField>(index) {
                field.type_text(text);
                continue;
            }
            if let Some(editor) = stack.child_mut::<TextEditor>(index) {
                editor.type_text(text);
                continue;
            }
            if let Some(editor) = stack.child_mut::<LargeTextEditor>(index) {
                editor.type_text(text);
                continue;
            }
            if let Some(form) = stack.child_mut::<Form>(index) {
                form.type_text(text);
                continue;
            }
            if let Some(table) = stack.child_mut::<BasicTable>(index) {
                table.type_text(text);
                continue;
            }
            if let Some(sidebar) = stack.child_mut::<Sidebar>(index) {
                sidebar.page_text(text);
            }
        }
    }

    fn forward_key(&mut self, key: Key) {
        let Some(stack) = self.stack_mut() else {
            return;
        };
        for index in 0..stack.len() {
            // TextEditor key handling needs a FontSystem, which the App
            // key callback does not provide, so editors only get text.
            if let Some(field) = stack.child_mut::<BasicTextField>(index) {
                field.key(key);
                continue;
            }
            if let Some(field) = stack.child_mut::<LargeTextField>(index) {
                field.key(key);
                continue;
            }
            if let Some(field) = stack.child_mut::<SecureField>(index) {
                field.key(key);
                continue;
            }
            if let Some(field) = stack.child_mut::<SearchField>(index) {
                field.key(key);
                continue;
            }
            if let Some(form) = stack.child_mut::<Form>(index) {
                form.key(key);
                continue;
            }
            if let Some(table) = stack.child_mut::<BasicTable>(index) {
                table.key(key);
                continue;
            }
            if let Some(picker) = stack.child_mut::<DatePicker>(index) {
                picker.key(key);
                continue;
            }
            if let Some(sidebar) = stack.child_mut::<Sidebar>(index) {
                sidebar.page_key(key);
            }
        }
    }

    fn wants_text_cursor(&mut self) -> bool {
        let Some(stack) = self.stack_mut() else {
            return false;
        };
        for index in 0..stack.len() {
            if let Some(form) = stack.child_mut::<Form>(index) {
                if form.wants_text_cursor() {
                    return true;
                }
            }
            if let Some(table) = stack.child_mut::<BasicTable>(index) {
                if table.wants_text_cursor() {
                    return true;
                }
            }
        }
        false
    }
}

impl App for AllElements {
    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
        viewport: Viewport,
        time_secs: f64,
    ) {
        self.watcher.poll(time_secs);
        self.watcher.set_focused(self.focused, time_secs);
        let palette = self.watcher.palette(time_secs);
        self.bg = palette.bg;
        let theme = self.watcher.theme();
        let dark = theme.mode == ThemeMode::Dark;
        let focused = self.focused;

        // Demo buttons open one overlay at a time.
        if std::mem::replace(&mut *self.show_ok.borrow_mut(), false) {
            self.alert_both.dismiss();
            self.alert_action.dismiss();
            self.alert_dialog.dismiss();
            self.alert_icon.dismiss();
            self.alert_ok.show();
        }
        if std::mem::replace(&mut *self.show_both.borrow_mut(), false) {
            self.alert_ok.dismiss();
            self.alert_action.dismiss();
            self.alert_dialog.dismiss();
            self.alert_icon.dismiss();
            self.alert_both.show();
        }
        if std::mem::replace(&mut *self.show_action.borrow_mut(), false) {
            self.alert_ok.dismiss();
            self.alert_both.dismiss();
            self.alert_dialog.dismiss();
            self.alert_icon.dismiss();
            self.alert_action.show();
        }
        if std::mem::replace(&mut *self.show_dialog.borrow_mut(), false) {
            self.alert_ok.dismiss();
            self.alert_both.dismiss();
            self.alert_action.dismiss();
            self.alert_icon.dismiss();
            self.alert_dialog.show();
        }
        if std::mem::replace(&mut *self.show_icon.borrow_mut(), false) {
            self.alert_ok.dismiss();
            self.alert_both.dismiss();
            self.alert_action.dismiss();
            self.alert_dialog.dismiss();
            self.alert_icon.show();
        }
        if let Some(size) = self.open_sheet.borrow_mut().take() {
            self.sheet.set_size(size);
            self.sheet.show();
        }
        if std::mem::replace(&mut *self.dismiss_sheet.borrow_mut(), false) {
            self.sheet.dismiss();
        }
        if std::mem::replace(&mut *self.show_picker.borrow_mut(), false) {
            self.picker.show();
        }
        // Deferred table edit from the context menu action.
        if self.table_edit.take() {
            let pending = self.table_pending.get();
            if let Some((row, col)) = pending {
                if let Some(table) = self.table_mut() {
                    table.begin_edit(row, col);
                }
            }
        }

        self.theme_page(
            palette.accent,
            dark,
            theme.mode,
            theme.glass,
            viewport,
            focused,
        );
        self.scroll.set_theme(palette.accent, dark);
        self.scroll.set_focused(focused);

        for alert in [&mut self.alert_ok, &mut self.alert_both] {
            alert.set_theme(theme.mode, palette.accent, theme.glass);
            alert.set_focused(focused);
        }
        self.alert_action
            .set_theme(theme.mode, palette.accent, theme.glass);
        self.alert_action.set_focused(focused);
        self.alert_dialog
            .set_theme(theme.mode, palette.accent, theme.glass);
        self.alert_dialog.set_focused(focused);
        self.alert_icon
            .set_theme(theme.mode, palette.text, theme.glass);
        self.alert_icon.set_focused(focused);
        self.sheet.set_theme(dark);
        self.sheet.set_focused(focused);
        if let Some(text) = self.sheet.child_mut().child_mut::<BasicText>(0) {
            text.set_theme(theme.mode);
            text.set_focused(focused);
        }
        if let Some(button) = self.sheet.child_mut().child_mut::<Button>(1) {
            button.set_theme(palette.accent, dark);
            button.set_focused(focused);
        }
        self.picker.set_theme(theme.mode, theme.glass);
        self.picker.set_focused(focused);
        self.context
            .set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
        self.context.set_theme(palette.accent, dark);
        self.context.set_glass(theme.mode, theme.glass);
        self.context.set_focused(focused);

        let status = self.form_status.borrow().clone();
        let selected = self.sidebar_selected.get();
        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_modal_blocked(self.any_overlay());
        self.bar.set_title(if status.is_empty() {
            format!("All Elements - sidebar page {selected}")
        } else {
            format!("All Elements - {status}")
        });
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let top = viewport.y + 31.0;
        let content_h = viewport.height - 31.0;
        self.context
            .set_area((viewport.x, top, viewport.width, content_h));
        self.scroll.place(
            fonts,
            viewport.x + 24.0,
            top + 16.0,
            viewport.width - 48.0,
            (viewport.height - 47.0).max(0.0),
        );
        self.scroll.draw(scene, fonts, images);

        // Modal overlays on top of the scrolled content.
        if self.alert_ok.is_visible() {
            self.alert_ok
                .set_viewport(viewport.x, top, viewport.width, content_h);
            self.alert_ok.draw(scene, fonts, images);
        } else if self.alert_both.is_visible() {
            self.alert_both
                .set_viewport(viewport.x, top, viewport.width, content_h);
            self.alert_both.draw(scene, fonts, images);
        } else if self.alert_action.is_visible() {
            self.alert_action
                .set_viewport(viewport.x, top, viewport.width, content_h);
            self.alert_action.draw(scene, fonts, images);
        } else if self.alert_dialog.is_visible() {
            self.alert_dialog
                .set_viewport(viewport.x, top, viewport.width, content_h);
            self.alert_dialog.draw(scene, fonts, images);
        } else if self.alert_icon.is_visible() {
            self.alert_icon
                .set_viewport(viewport.x, top, viewport.width, content_h);
            self.alert_icon.draw(scene, fonts, images);
        }
        if self.sheet.is_visible() {
            self.sheet
                .set_viewport(viewport.x, top, viewport.width, content_h);
            self.sheet.draw(scene, fonts, images);
        }
        if self.picker.is_visible() {
            self.picker
                .set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
            self.picker.draw(scene, fonts, images);
        }
        if self.context.is_open() {
            self.context.place(
                fonts,
                viewport.x,
                viewport.y,
                viewport.width,
                viewport.height,
            );
            self.context.draw(scene, fonts, images);
        }
        let text_cursor = self.wants_text_cursor();
        self.text_cursor.set(text_cursor);
    }

    fn background(&self) -> Color {
        self.bg
    }

    fn wants_backdrop(&self) -> bool {
        // Glass, materials and picker panels need the blur pass.
        true
    }

    fn drag_region(&self) -> Option<(f32, f32, f32, f32)> {
        Some(self.bar.drag_rect())
    }

    fn poll_window_command(&mut self) -> Option<WindowCommand> {
        self.command.take()
    }

    fn cursor(&self, _x: f64, _y: f64) -> CursorKind {
        if self.text_cursor.get() {
            CursorKind::Text
        } else {
            CursorKind::Default
        }
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        if self.alert_ok.is_visible() {
            self.alert_ok.mouse_down(x, y);
            return;
        }
        if self.alert_both.is_visible() {
            self.alert_both.mouse_down(x, y);
            return;
        }
        if self.alert_action.is_visible() {
            self.alert_action.mouse_down(x, y);
            return;
        }
        if self.alert_dialog.is_visible() {
            self.alert_dialog.mouse_down(x, y);
            return;
        }
        if self.alert_icon.is_visible() {
            self.alert_icon.mouse_down(x, y);
            return;
        }
        if self.sheet.is_visible() {
            self.sheet.mouse_down(x, y);
            return;
        }
        if self.picker.is_visible() {
            self.picker.mouse_down(x, y);
            return;
        }
        if self.context.is_open() {
            self.context.mouse_down(x, y);
            return;
        }
        match self.bar.press(x as f32, y as f32) {
            Some(TrafficAction::Close) => self.command = Some(WindowCommand::Close),
            Some(TrafficAction::Minimize) => self.command = Some(WindowCommand::Minimize),
            Some(TrafficAction::Maximize) => {
                self.command = Some(WindowCommand::ToggleMaximize)
            }
            None => self.scroll.mouse_down(x, y),
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        if self.alert_ok.is_visible() {
            match self.alert_ok.mouse_up(x, y) {
                Some(AlertAction::Ok) | Some(AlertAction::Cancel) => {
                    self.alert_ok.dismiss();
                }
                None => {}
            }
            return;
        }
        if self.alert_both.is_visible() {
            match self.alert_both.mouse_up(x, y) {
                Some(AlertAction::Ok) | Some(AlertAction::Cancel) => {
                    self.alert_both.dismiss();
                }
                None => {}
            }
            return;
        }
        if self.alert_action.is_visible() {
            if self.alert_action.mouse_up(x, y).is_some() {
                self.alert_action.dismiss();
            }
            return;
        }
        if self.alert_dialog.is_visible() {
            if self.alert_dialog.mouse_up(x, y).is_some() {
                self.alert_dialog.dismiss();
            }
            return;
        }
        if self.alert_icon.is_visible() {
            if self.alert_icon.mouse_up(x, y).is_some() {
                self.alert_icon.dismiss();
            }
            return;
        }
        if self.sheet.is_visible() {
            self.sheet.mouse_up(x, y);
            return;
        }
        if self.picker.is_visible() {
            self.picker.mouse_up(x, y);
            if !self.picker.is_visible() {
                // Closed by the popup itself; nothing to do.
            }
            return;
        }
        if self.context.is_open() {
            self.context.mouse_up(x, y);
            return;
        }
        self.scroll.mouse_up(x, y);
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
        if self.any_overlay() {
            if self.picker.is_visible() {
                self.picker.mouse_move(x, y);
            }
            if self.context.is_open() {
                self.context.mouse_move(x, y);
            }
            return;
        }
        self.scroll.mouse_move(x, y);
    }

    fn mouse_wheel(&mut self, dx: f64, dy: f64) {
        if self.context.is_open() {
            self.context.mouse_wheel(dx, dy);
        } else if !self.any_overlay() {
            self.scroll.mouse_wheel(dx, dy);
        }
    }

    fn context_click(&mut self, x: f64, y: f64) {
        let hit = self.table_mut().and_then(|table| table.cell_at(x, y));
        match hit {
            Some(TableHit::Cell(row, col)) => {
                let editable = self
                    .table_mut()
                    .map(|table| table.is_cell_editable(row, col))
                    .unwrap_or(false);
                if editable {
                    self.table_pending.set(Some((row, col)));
                    self.context.context_click(x, y);
                } else {
                    self.context.close();
                }
            }
            _ => self.context.close(),
        }
    }

    fn text(&mut self, text: &str) {
        if self.picker.is_visible() {
            self.picker.type_text(text);
            return;
        }
        self.forward_text(text);
    }

    fn key(&mut self, key: Key) {
        if self.sheet.is_visible() {
            self.sheet.key(key);
        }
        if self.picker.is_visible() && self.picker.key(key) {
            return;
        }
        self.forward_key(key);
    }

    fn set_modifiers(&mut self, ctrl: bool, shift: bool) {
        if let Some(table) = self.table_mut() {
            table.set_modifiers(ctrl, shift);
        }
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("All Elements", 1000, 800, AllElements::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
