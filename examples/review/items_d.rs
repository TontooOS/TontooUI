//! Review items: TabView (22), View (10).

use super::ReviewItem;
use tontooui::prelude::*;
use tontooui::ManageSubscriptionsSheet;

pub fn items(out: &mut Vec<ReviewItem>) {
    // ── TabView (22) ──
    out.push(ReviewItem { id: "tab_section", category: "TabView", title: "TabSection", desc: "A container that you can use to add hierarchy within a tab view.", badge: "initializer", make: || WidgetNode::new(TabSection::new()) });
    out.push(ReviewItem { id: "tab_bar_only", category: "TabView", title: "TabBarOnlyTabViewStyle", desc: "A tab view style that displays a tab bar when possible.", badge: "initializer", make: || WidgetNode::new(TabBarOnlyTabViewStyle::new()) });
    out.push(ReviewItem { id: "tab_view", category: "TabView", title: "TabView", desc: "Creates Tabs with title, image, systemImage and custom Label.", badge: "initializer", make: || WidgetNode::new(TabView::new()) });
    out.push(ReviewItem { id: "tab_search_role", category: "TabView", title: "Search Tab Role", desc: "On Liquid Glass, this tab is placed in its own group.", badge: "initializer", make: || WidgetNode::new(SearchTabRole::new()) });
    out.push(ReviewItem { id: "tab_grouped", category: "TabView", title: "GroupedTabViewStyle", desc: "A tab view style that displays a tab bar that groups its tabs together.", badge: "style", make: || WidgetNode::new(GroupedTabViewStyle::new()) });
    out.push(ReviewItem { id: "tab_page", category: "TabView", title: "PageTabViewStyle", desc: "A TabViewStyle that displays a paged scrolling TabView.", badge: "style", make: || WidgetNode::new(PageTabViewStyle::new()) });
    out.push(ReviewItem { id: "tab_vertical_page", category: "TabView", title: "VerticalPageTabViewStyle", desc: "A TabViewStyle that displays a vertical TabView interaction and appearance.", badge: "style", make: || WidgetNode::new(VerticalPageTabViewStyle::new()) });
    out.push(ReviewItem { id: "tab_sidebar_adaptable", category: "TabView", title: "SidebarAdaptableTabViewStyle", desc: "A tab bar style that adapts to each platform.", badge: "style", make: || WidgetNode::new(SidebarAdaptableTabViewStyle::new()) });
    out.push(ReviewItem { id: "tab_bottom_placement", category: "TabView", title: "TabView Bottom Accessory Placement", desc: "A placement of the bottom accessory in a tab view.", badge: "style", make: || WidgetNode::new(TabViewBottomAccessoryPlacement::new()) });
    out.push(ReviewItem { id: "tab_collapsed", category: "TabView", title: "Default Collapsed Tab Section", desc: "Sets the default expansion state for the section containing this tab to collapsed.", badge: "modifier", make: || WidgetNode::new(DefaultCollapsedTabSection::new()) });
    out.push(ReviewItem { id: "tab_custom_behavior", category: "TabView", title: "Tab View Customization Behavior", desc: "Configures the customization behavior of customizable tab view content.", badge: "modifier", make: || WidgetNode::new(TabViewCustomizationBehavior::new()) });
    out.push(ReviewItem { id: "tab_custom", category: "TabView", title: "Tab View Customization", desc: "Specifies the customizations to apply to the sidebar representation of the tab.", badge: "modifier", make: || WidgetNode::new(TabViewCustomization::new()) });
    out.push(ReviewItem { id: "tab_sidebar_footer", category: "TabView", title: "Tab View Side Bar Footer", desc: "Adds a custom footer to the sidebar of a tab view.", badge: "modifier", make: || WidgetNode::new(TabViewSideBarFooter::new()) });
    out.push(ReviewItem { id: "tab_sidebar_bottom", category: "TabView", title: "Tab View Side Bar Bottom Bar", desc: "Adds a custom bottom bar to the sidebar of a tab view.", badge: "modifier", make: || WidgetNode::new(TabViewSideBarBottomBar::new()) });
    out.push(ReviewItem { id: "tab_adaptable_placement", category: "TabView", title: "Default Adaptable Tab Bar Placement", desc: "Specifies the default placement for the tabs in a tab view using the adaptable style.", badge: "modifier", make: || WidgetNode::new(DefaultAdaptableTabBarPlacement::new()) });
    out.push(ReviewItem { id: "tab_section_actions", category: "TabView", title: "Tab Bar Section Actions", desc: "Adds custom actions to a tab section.", badge: "modifier", make: || WidgetNode::new(TabBarSectionActions::new()) });
    out.push(ReviewItem { id: "tab_sidebar_header", category: "TabView", title: "Tab View Side Bar Header", desc: "Adds a custom header to the sidebar of a tab view.", badge: "modifier", make: || WidgetNode::new(TabViewSideBarHeader::new()) });
    out.push(ReviewItem { id: "tab_badge", category: "TabView", title: "Tab Badge", desc: "Generates a badge for a tab from an integer value.", badge: "modifier", make: || WidgetNode::new(TabBadge::new()) });
    out.push(ReviewItem { id: "tab_hidden_index", category: "TabView", title: "Hidden Index PageTabViewStyle", desc: "A TabViewStyle that displays a paged scrolling TabView with a hidden index.", badge: "modifier", make: || WidgetNode::new(HiddenIndexPageTabViewStyle::new()) });
    out.push(ReviewItem { id: "tab_value", category: "TabView", title: "Value Tab View", desc: "Creates a tab view that uses a builder to specify selection values.", badge: "modifier", make: || WidgetNode::new(ValueTabView::new()) });
    out.push(ReviewItem { id: "tab_hide_scroll", category: "TabView", title: "Hide Tab Bar On Scroll Down", desc: "Minimize the tab bar when downwards scrolling starts.", badge: "modifier", make: || WidgetNode::new(HideTabBarOnScrollDown::new()) });
    out.push(ReviewItem { id: "tab_bottom_accessory", category: "TabView", title: "Bottom Accessory", desc: "A modifier to place content above the tabs.", badge: "modifier", make: || WidgetNode::new(BottomAccessory::new()) });

    // ── View (10) ──
    out.push(ReviewItem { id: "view_music_picker", category: "View", title: "Music Picker", desc: "Presents a music picker to select items from the Apple Music catalog.", badge: "modifier", make: || WidgetNode::new(MusicPicker::new()) });
    out.push(ReviewItem { id: "view_appstore", category: "View", title: "App Store Overlay", desc: "Presents a StoreKit overlay when a given condition is true.", badge: "modifier", make: || WidgetNode::new(AppStoreOverlay::new("com.example.app")) });
    out.push(ReviewItem { id: "view_subscriptions", category: "View", title: "Manage Subscriptions Sheet", desc: "Opens the manage subscriptions sheet.", badge: "modifier", make: || WidgetNode::new(ManageSubscriptionsSheet::new()) });
    out.push(ReviewItem { id: "view_swipe_container", category: "View", title: "Swipe Container", desc: "Only allows a single active swipe within a container.", badge: "modifier", make: || WidgetNode::new(SwipeContainer::new()) });
    out.push(ReviewItem { id: "view_swipe_action", category: "View", title: "Swipe Action", desc: "Adds custom swipe actions to a row in a list or container.", badge: "modifier", make: || WidgetNode::new(SwipeAction::new()) });
    out.push(ReviewItem { id: "view_split_bg", category: "View", title: "Navigation Split View Background", desc: "A background placement behind the content of a NavigationSplitView.", badge: "modifier", make: || WidgetNode::new(NavigationSplitViewBackground::new()) });
    out.push(ReviewItem { id: "view_container_bg", category: "View", title: "Navigation Container Background", desc: "Sets the container background of the enclosing container using a view.", badge: "modifier", make: || WidgetNode::new(NavigationContainerBackground::new()) });
    out.push(ReviewItem { id: "view_control_size", category: "View", title: "ControlSize", desc: "A control version that is the default size.", badge: "modifier", make: || WidgetNode::new(ControlSizeView::new()) });
    out.push(ReviewItem { id: "view_bg_extension", category: "View", title: "Background Extension Effect", desc: "Adds the background extension effect to the view.", badge: "modifier", make: || WidgetNode::new(BackgroundExtensionEffect::new()) });
    out.push(ReviewItem { id: "view_glass", category: "View", title: "Glass effect", desc: "Applies the Liquid Glass effect to a view.", badge: "modifier", make: || WidgetNode::new(GlassEffect::new()) });
}
