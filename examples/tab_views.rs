//! TontooUI TabView demo — all 22 TabView elements directly on window.

use tontooui::prelude::*;
use tontooui::{
    BottomAccessory, DefaultAdaptableTabBarPlacement, DefaultCollapsedTabSection,
    GroupedTabViewStyle, HiddenIndexPageTabViewStyle, HideTabBarOnScrollDown, PageTabViewStyle,
    SearchTabRole, SidebarAdaptableTabViewStyle, TabBadge, TabBarOnlyTabViewStyle,
    TabBarSectionActions, TabSection, TabView, TabViewBottomAccessoryPlacement,
    TabViewCustomization, TabViewCustomizationBehavior, TabViewSideBarBottomBar,
    TabViewSideBarFooter, TabViewSideBarHeader, ValueTabView, VerticalPageTabViewStyle,
};

fn cell(title: &str, desc: &str, badge: &str, preview: impl Widget + 'static) -> impl Widget {
    let is_dark = uikit::app::ColorScheme::detect_system() == uikit::app::ColorScheme::Dark;
    let title_c = if is_dark { Color::WHITE } else { Color::from_hex("#1d1d1d").unwrap() };
    let desc_c = if is_dark { Color::from_hex("#98989d").unwrap() } else { Color::from_hex("#6c6c70").unwrap() };
    VStack::new()
        .spacing(8.0)
        .child(
            HStack::new().spacing(0.0).child(
                Text::new(badge).font_size(7.0).bold().color(Color::from_hex("#0A84FF").unwrap()),
            ),
        )
        .child(preview)
        .child(Text::new(title).font_size(11.0).bold().color(title_c).max_width(200.0))
        .child(Text::new(desc).font_size(9.0).color(desc_c).max_width(200.0))
}

fn main() {
    let mut app = App::new("TontooUI TabView", 1220, 1100);
    let title = Text::new("TabView").font_size(18.0).bold().color(Color::from_hex("#7dd3e0").unwrap());

    let row1 = HStack::new().spacing(24.0)
        .child(cell("TabSection", "A container that you can use to add hierarchy within a tab view.", "initializer", TabSection::new()))
        .child(cell("TabBarOnlyTabViewStyle", "A tab view style that displays a tab bar when possible.", "initializer", TabBarOnlyTabViewStyle::new()))
        .child(cell("TabView", "Creates Tabs with title, image, systemImage and custom Label.", "initializer", TabView::new()))
        .child(cell("Search Tab Role", "On Liquid Glass, this tab is placed in its own group. In left-to-right (LTR) layout...", "initializer", SearchTabRole::new()));

    let row2 = HStack::new().spacing(24.0)
        .child(cell("GroupedTabViewStyle", "A tab view style that displays a tab bar that groups its tabs together.", "style", GroupedTabViewStyle::new()))
        .child(cell("PageTabViewStyle", "A TabViewStyle that displays a paged scrolling TabView.", "style", PageTabViewStyle::new()))
        .child(cell("VerticalPageTabViewStyle", "A TabViewStyle that displays a vertical TabView interaction and appearance.", "style", VerticalPageTabViewStyle::new()))
        .child(cell("SidebarAdaptableTabViewStyle", "A tab bar style that adapts to each platform.", "style", SidebarAdaptableTabViewStyle::new()));

    let row3 = HStack::new().spacing(24.0)
        .child(cell("TabView Bottom Accessory Plac...", "A placement of the bottom accessory in a tab view. You can use this to adjust...", "style", TabViewBottomAccessoryPlacement::new()))
        .child(cell("Default Collapsed Tab Section", "Sets the default expansion state for the section containing this tab to collapse...", "modifier", DefaultCollapsedTabSection::new()))
        .child(cell("Tab View Customization Behavior", "Configures the customization behavior of customizable tab view content.", "modifier", TabViewCustomizationBehavior::new()))
        .child(cell("Tab View Customization", "Specifies the customizations to apply to the sidebar representation of the tab...", "modifier", TabViewCustomization::new()));

    let row4 = HStack::new().spacing(24.0)
        .child(cell("Tab View Side Bar Footer", "Adds a custom footer to the sidebar of a tab view.", "modifier", TabViewSideBarFooter::new()))
        .child(cell("Tab View Side Bar Bottom Bar", "Adds a custom bottom bar to the sidebar of a tab view.", "modifier", TabViewSideBarBottomBar::new()))
        .child(cell("Default Adaptable Tab Bar Place...", "Specifies the default placement for the tabs in a tab view using the adaptable...", "modifier", DefaultAdaptableTabBarPlacement::new()))
        .child(cell("Tab Bar Section Actions", "Adds custom actions to a tab section.", "modifier", TabBarSectionActions::new()));

    let row5 = HStack::new().spacing(24.0)
        .child(cell("Tab View Side Bar Header", "Adds a custom header to the sidebar of a tab view.", "modifier", TabViewSideBarHeader::new()))
        .child(cell("Tab Badge", "Generates a badge for a tab from an integer value.", "modifier", TabBadge::new()))
        .child(cell("Hidden Index PageTabViewStyle", "A TabViewStyle that displays a paged scrolling TabView with a hidden index.", "modifier", HiddenIndexPageTabViewStyle::new()))
        .child(cell("Value Tab View", "Creates a tab view that uses a builder to create and specify selection values for...", "modifier", ValueTabView::new()));

    let row6 = HStack::new().spacing(24.0)
        .child(cell("Hide Tab Bar On Scroll Down", "Minimize the tab bar when downwards scrolling starts. Minimizing is supporte...", "modifier", HideTabBarOnScrollDown::new()))
        .child(cell("Bottom Accessory", "A modifier to place content above the tabs", "modifier", BottomAccessory::new()));

    let grid = VStack::new().spacing(28.0).child(row1).child(row2).child(row3).child(row4).child(row5).child(row6);
    let root = VStack::new().spacing(18.0).child(title).child(grid);
    app.set_root(root);
    app.run();
}
