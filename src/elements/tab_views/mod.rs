//! TabView — category for tab view containers and styles.
//!
//! Category `TabView` groups tab view containers, styles, and modifiers.
//! All elements render directly on the window background (#1d1d1d dark / #ececec light),
//! no extra card, SF Pro.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`tab_section`] | [`TabSection`] | `initializer` | A container that you can use to add hierarchy within a tab view. |
//! | [`tab_bar_only_style`] | [`TabBarOnlyTabViewStyle`] | `initializer` | A tab view style that displays a tab bar when possible. |
//! | [`tab_view`] | [`TabView`] | `initializer` | Creates Tabs with title, image, systemImage and custom Label. |
//! | [`search_tab_role`] | [`SearchTabRole`] | `initializer` | On Liquid Glass, this tab is placed in its own group. In left-to-right (LTR) layout... |
//! | [`grouped_style`] | [`GroupedTabViewStyle`] | `style` | A tab view style that displays a tab bar that groups its tabs together. |
//! | [`page_style`] | [`PageTabViewStyle`] | `style` | A TabViewStyle that displays a paged scrolling TabView. |
//! | [`vertical_page_style`] | [`VerticalPageTabViewStyle`] | `style` | A TabViewStyle that displays a vertical TabView interaction and appearance. |
//! | [`sidebar_adaptable_style`] | [`SidebarAdaptableTabViewStyle`] | `style` | A tab bar style that adapts to each platform. |
//! | [`bottom_accessory_placement`] | [`TabViewBottomAccessoryPlacement`] | `style` | A placement of the bottom accessory in a tab view. You can use this to adjust... |
//! | [`default_collapsed_section`] | [`DefaultCollapsedTabSection`] | `modifier` | Sets the default expansion state for the section containing this tab to collapse... |
//! | [`customization_behavior`] | [`TabViewCustomizationBehavior`] | `modifier` | Configures the customization behavior of customizable tab view content. |
//! | [`customization`] | [`TabViewCustomization`] | `modifier` | Specifies the customizations to apply to the sidebar representation of the tab... |
//! | [`side_bar_footer`] | [`TabViewSideBarFooter`] | `modifier` | Adds a custom footer to the sidebar of a tab view. |
//! | [`side_bar_bottom_bar`] | [`TabViewSideBarBottomBar`] | `modifier` | Adds a custom bottom bar to the sidebar of a tab view. |
//! | [`default_adaptable_placement`] | [`DefaultAdaptableTabBarPlacement`] | `modifier` | Specifies the default placement for the tabs in a tab view using the adaptable... |
//! | [`section_actions`] | [`TabBarSectionActions`] | `modifier` | Adds custom actions to a tab section. |
//! | [`side_bar_header`] | [`TabViewSideBarHeader`] | `modifier` | Adds a custom header to the sidebar of a tab view. |
//! | [`badge`] | [`TabBadge`] | `modifier` | Generates a badge for a tab from an integer value. |
//! | [`hidden_index_page_style`] | [`HiddenIndexPageTabViewStyle`] | `modifier` | A TabViewStyle that displays a paged scrolling TabView with a hidden index. |
//! | [`value_tab_view`] | [`ValueTabView`] | `modifier` | Creates a tab view that uses a builder to create and specify selection values for... |
//! | [`hide_on_scroll`] | [`HideTabBarOnScrollDown`] | `modifier` | Minimize the tab bar when downwards scrolling starts. Minimizing is supporte... |
//! | [`bottom_accessory`] | [`BottomAccessory`] | `modifier` | A modifier to place content above the tabs |

pub mod tab_section;
pub mod tab_bar_only_style;
pub mod tab_view;
pub mod search_tab_role;
pub mod grouped_style;
pub mod page_style;
pub mod vertical_page_style;
pub mod sidebar_adaptable_style;
pub mod bottom_accessory_placement;
pub mod default_collapsed_section;
pub mod customization_behavior;
pub mod customization;
pub mod side_bar_footer;
pub mod side_bar_bottom_bar;
pub mod default_adaptable_placement;
pub mod section_actions;
pub mod side_bar_header;
pub mod badge;
pub mod hidden_index_page_style;
pub mod value_tab_view;
pub mod hide_on_scroll;
pub mod bottom_accessory;

pub use tab_section::TabSection;
pub use tab_bar_only_style::TabBarOnlyTabViewStyle;
pub use tab_view::TabView;
pub use search_tab_role::SearchTabRole;
pub use grouped_style::GroupedTabViewStyle;
pub use page_style::PageTabViewStyle;
pub use vertical_page_style::VerticalPageTabViewStyle;
pub use sidebar_adaptable_style::SidebarAdaptableTabViewStyle;
pub use bottom_accessory_placement::TabViewBottomAccessoryPlacement;
pub use default_collapsed_section::DefaultCollapsedTabSection;
pub use customization_behavior::TabViewCustomizationBehavior;
pub use customization::TabViewCustomization;
pub use side_bar_footer::TabViewSideBarFooter;
pub use side_bar_bottom_bar::TabViewSideBarBottomBar;
pub use default_adaptable_placement::DefaultAdaptableTabBarPlacement;
pub use section_actions::TabBarSectionActions;
pub use side_bar_header::TabViewSideBarHeader;
pub use badge::TabBadge;
pub use hidden_index_page_style::HiddenIndexPageTabViewStyle;
pub use value_tab_view::ValueTabView;
pub use hide_on_scroll::HideTabBarOnScrollDown;
pub use bottom_accessory::BottomAccessory;
