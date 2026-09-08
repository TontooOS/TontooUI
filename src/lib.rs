//! # TontooUI
//!
//! A SwiftUI-inspired declarative UI layer for TontooOS.
//!
//! Built on top of [`TontooUIKit`](https://docs.rs/uikit) — provides
//! pre-made elements with a clean, SwiftUI-like builder API.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use tontooui::prelude::*;
//!
//! fn main() {
//!     let mut app = App::new("My App", 800, 600);
//!     app.set_root(
//!         VStack::new()
//!             .spacing(8.0)
//!             .child(Text::new("Hello, TontooOS!").font_size(24.0).bold())
//!             .child(TextInput::new("Enter text...")
//!                 .on_change(|text| println!("Changed: {}", text)))
//!     );
//!     app.run();
//! }
//! ```
//!
//! ## Elements
//!
//! | Element | Description |
//! |---|---|
//! | [`Button`] | SwiftUI-style push button: roles, styles, tint, border shapes, sizing |
//! | [`Toolbar`] | Glass capsule toolbar bar with items and spacers |
//! | [`Toggle`] | SwiftUI-style toggle: switch style and checkbox style |
//! | [`RenameButton`] | System button that triggers a standard rename action |
//! | [`EditButton`] | System button that toggles between Edit and Done |
//! | [`PasteButton`] | System button that reads text from the clipboard |
//! | [`TextInput`] | Single-line text input field |
//! | [`WheelPicker`] | macOS/iOS style scroll wheel picker |
//! | [`Picker`] | SwiftUI-style picker: all picker styles (wheel, segmented, palette, radio, menu, inline, tabs, navigation) + sections/dividers |
//! | [`PickerItem`] | Single option inside a `Picker` (title / image / systemImage / custom label) |
//! | [`PickerSection`] | Section inside a `Picker` |
//! | [`PickerStyle`] | Picker presentation style enum |
//! | [`Slider`] | Slider with spring physics and squish animation |
//! | [`ProgressView`] | Loading indicator: spinner/ring or linear bar |
//! | [`ProgressViewStyle`] | ProgressView style (Circular / Linear) |
//! | [`Sidebar`] | macOS-style sidebar with traffic lights, search, and item list |
//! | [`ContentUnavailableView`] | Empty state with icon, title and hint message |
//! | [`Gauge`] | SwiftUI-style gauge: linear/circular, capacity/marker, accessory styles, tint/gradient |
//! | [`GaugeStyle`] | Gauge presentation style enum |
//! | [`GaugeTint`] | Gauge tint (single color or gradient) |
//! | [`Menu`] | SwiftUI-style menu with Liquid Glass (items, dividers, sections, nested) |
//! | [`ContextMenu`] | Context menu (secondary gesture) with optional custom preview |
//! | [`ViewThatFits`] | Adaptive container that picks first child fitting available space |
//! | [`ViewThatFitsAxis`] | Axis for ViewThatFits (Horizontal/Vertical/Both) |
//! | [`Divider`] | SwiftUI-style separator line (Light/Dark, SF Pro context) |
//! | [`DividerOrientation`] | Divider orientation (Horizontal/Vertical) |
//! | [`List`] | SwiftUI-style list (sections, rows, disclosure, outline) |
//! | [`ListSection`] | Section with header/footer and rows |
//! | [`ListRow`] | Row with label, badge, tint, separators, swipe actions |
//! | [`ListStyle`] | List style (Plain/Inset/InsetGrouped/Sidebar/Elliptical/Carousel/Bordered) |
//! | [`OutlineGroup`] | Hierarchical disclosure outline |
//! | [`DisclosureGroup`] | Collapsible disclosure |
//! | [`ColorOpacity`] | Style: Color opacity modifier (5 swatches 1.0→0.12) |
//! | [`ColorGradient`] | Modifier: Color gradient (interpolated swatches + bar) |
//! | [`ColorVariants`] | Modifier: HierarchicalShapeStyle variants (Primary/Secondary/Tertiary/Quaternary) |
//! | [`HierarchicalVariant`] | Variant level for ColorVariants |
//! | [`UIKitSeparatorColors`] | Type: UIKit separator palette (separator / opaqueSeparator) |
//! | [`UIKitSeparatorColor`] | Enum for single separator color |
//! | [`UIKitContentBackgroundColors`] | Type: UIKit content background palette (6 system backgrounds) |
//! | [`UIKitBackgroundColor`] | Enum for single background color |
//! | [`UIKitTextColors`] | Type: UIKit text palette (placeholderText) |
//! | [`UIKitTextColor`] | Enum for single text color |
//! | [`UIKitFillColors`] | Type: UIKit fill palette (systemFill ×4) |
//! | [`UIKitFillColor`] | Enum for single fill color |
//! | [`UIKitLabelColors`] | Type: UIKit label palette (label ×4) |
//! | [`UIKitLabelColor`] | Enum for single label color |
//! | [`SemanticColors`] | Type: Semantic palette (primary/secondary) |
//! | [`SemanticColor`] | Enum for single semantic color |
//! | [`StandardColors`] | Type: Standard SwiftUI palette (12 colors grid) |
//! | [`StandardColor`] | Enum for single standard color |
//! | [`TextFormat`] | Initializer: Text that displays formatted representation of non-string type |
//! | [`TextFormatKind`] | Format kind enum for TextFormat |
//! | [`Materials`] | Type: All materials (ultraThinMaterial … ultraThickMaterial, bar) |
//! | [`Material`] | Enum for single material thickness |
//! | [`HelpLink`] | Initializer: Button that opens app-specific help |
//! | [`TextFieldLink`] | Initializer: Control that requests text input when pressed |
//! | [`CustomPreviewShareLink`] | Initializer: ShareLink with custom label/preview |
//! | [`ShareLink`] | Initializer: View that controls a sharing presentation |
//! | [`Link`] | Initializer: Control for navigating to a URL |
//! | [`ControlGroup`] | Initializer: Creates a new ControlGroup with the specified children |
//! | [`ControlGroupStyle`] | Style enum for ControlGroup |
//! | [`PaletteControlGroupStyle`] | Style: presents content as a palette |
//! | [`NavigationControlGroupStyle`] | Style: the navigation control group style |
//! | [`MenuControlGroupStyle`] | Style: presents content as a menu |
//! | [`CompactMenuControlGroupStyle`] | Style: presents content as a compact menu |
//! | [`NavigationSubtitle`] | Modifier: Configures the view's subtitle for navigation |
//! | [`TabSection`] | Initializer: A container that you can use to add hierarchy within a tab view. |
//! | [`TabBarOnlyTabViewStyle`] | Initializer: A tab view style that displays a tab bar when possible. |
//! | [`TabView`] | Initializer: Creates Tabs with title, image, systemImage and custom Label. |
//! | [`SearchTabRole`] | Initializer: On Liquid Glass, this tab is placed in its own group. |
//! | [`GroupedTabViewStyle`] | Style: A tab view style that displays a tab bar that groups its tabs together. |
//! | [`PageTabViewStyle`] | Style: A TabViewStyle that displays a paged scrolling TabView. |
//! | [`VerticalPageTabViewStyle`] | Style: A TabViewStyle that displays a vertical TabView interaction and appearance. |
//! | [`SidebarAdaptableTabViewStyle`] | Style: A tab bar style that adapts to each platform. |
//! | [`TabViewBottomAccessoryPlacement`] | Style: A placement of the bottom accessory in a tab view. |
//! | [`DefaultCollapsedTabSection`] | Modifier: Sets the default expansion state for the section containing this tab to collapse... |
//! | [`TabViewCustomizationBehavior`] | Modifier: Configures the customization behavior of customizable tab view content. |
//! | [`TabViewCustomization`] | Modifier: Specifies the customizations to apply to the sidebar representation of the tab... |
//! | [`TabViewSideBarFooter`] | Modifier: Adds a custom footer to the sidebar of a tab view. |
//! | [`TabViewSideBarBottomBar`] | Modifier: Adds a custom bottom bar to the sidebar of a tab view. |
//! | [`DefaultAdaptableTabBarPlacement`] | Modifier: Specifies the default placement for the tabs in a tab view using the adaptable... |
//! | [`TabBarSectionActions`] | Modifier: Adds custom actions to a tab section. |
//! | [`TabViewSideBarHeader`] | Modifier: Adds a custom header to the sidebar of a tab view. |
//! | [`TabBadge`] | Modifier: Generates a badge for a tab from an integer value. |
//! | [`HiddenIndexPageTabViewStyle`] | Modifier: A TabViewStyle that displays a paged scrolling TabView with a hidden index. |
//! | [`ValueTabView`] | Modifier: Creates a tab view that uses a builder to create and specify selection values for... |
//! | [`HideTabBarOnScrollDown`] | Modifier: Minimize the tab bar when downwards scrolling starts. |
//! | [`BottomAccessory`] | Modifier: A modifier to place content above the tabs |
//! | [`MusicPicker`] | Modifier: Presents a music picker to select items from the Apple Music catalog |
//! | [`AppStoreOverlay`] | Modifier: Presents a StoreKit overlay when a given condition is true |
//! | [`ManageSubscriptionsSheet`] | Modifier: Opens the manage subscriptions sheet |
//! | [`SwipeContainer`] | Modifier: Only allows a single active swipe within a container |
//! | [`SwipeAction`] | Modifier: Adds custom swipe actions to a row in a list or container |
//! | [`NavigationSplitViewBackground`] | Modifier: A background placement behind the content of a NavigationSplitView |
//! | [`NavigationContainerBackground`] | Modifier: Sets the container background of the enclosing container using a view |
//! | [`ControlSizeView`] | Modifier: A control version that is the default size |
//! | [`BackgroundExtensionEffect`] | Modifier: Adds the background extension effect to the view |
//! | [`GlassEffect`] | Modifier: Applies the Liquid Glass effect to a view |
//! | [`SheetPlacement`] | Modifier: Sets the placement of a presentation within the presenting view |
//! | [`DisableSheetDismissSwipe`] | Modifier: Conditionally prevents interactive dismissal of presentations |
//! | [`PageScreenSheetSize`] | Modifier: Page sizing for devices smaller than a page of paper |
//! | [`FittedSheetSizing`] | Modifier: Sets the sizing of the containing presentation |
//! | [`SheetCornerRadius`] | Modifier: Requests a specific corner radius for the presentation |
//! | [`PrioritizeSheetContentScrolling`] | Modifier: Configure swipe gesture behavior on a presentation |
//! | [`SheetBackgroundInteraction`] | Modifier: Controls interaction with the view behind a presentation |
//! | [`SheetBackground`] | Modifier: Sets the presentation background of the enclosing sheet |
//! | [`SheetDragIndicatorVisibility`] | Modifier: Sets the visibility of the drag indicator on a sheet |
//! | [`SheetSize`] | Modifier: Sets the available detents for the enclosing sheet |
//! | [`ItemSheet`] | Modifier: Presents a sheet using the given item as data source |
//! | [`BooleanSheet`] | Modifier: Presents a sheet when a Boolean binding is true |
//! | [`Circle`] | Shape: A circle centered in its frame |
//! | [`Ellipse`] | Shape: An elliptical shape filling its frame |
//! | [`Capsule`] | Shape: A capsule (stadium) shape |
//! | [`RectangleShape`] | Shape: A rectangular shape filling its frame |
//! | [`RoundedRectangle`] | Shape: A rectangle with rounded corners |
//! | [`UnevenRoundedRectangle`] | Shape: A rectangle with uneven corner radii |
//! | [`ContainerRelativeShape`] | Shape: Inset version of the current container shape |
//! | [`CustomLabel`] | Initializer: Creates a label with a custom title and icon |
//! | [`ImageLabel`] | Initializer: Creates a label with an icon image and localized title |
//! | [`SystemImageLabel`] | Initializer: Creates a label with a system icon and localized title |
//! | [`LabelStyles`] | Style: Sets the style for labels within this view |
//! | [`CustomLabeledContent`] | Initializer: Standard labeled element with a custom value view |
//! | [`FormattedLabeledContent`] | Initializer: Labeled informational view from a formatted value |
//! | [`LabeledContent`] | Initializer: Creates a labeled informational view |
//! | [`UniformConcentricRectangle`] | Initializer: Rectangle with the same corner style on four corners |
//! | [`ConcentricRectangle`] | Initializer: Concentric rectangle with radii from the same circle |
//! | [`CustomPlaceholderAsyncImage`] | Initializer: Modifiable async image with custom placeholder |
//! | [`CustomPhasesAsyncImage`] | Initializer: Modifiable async image with custom phases |
//! | [`AsyncURLImage`] | Initializer: Loads and displays an image from a URL request |
//! | [`CustomSessionAsyncImage`] | Modifier: Adds a URL session for async images |
//! | [`PlaceholderIconProductView`] | Initializer: Product view with placeholder icon |
//! | [`CustomIconProductView`] | Initializer: Product view with custom icon |
//! | [`ProductViewElement`] | Initializer: Loads and merchandises an App Store product |
//! | [`CompactProductViewStyle`] | Style: Compact product view for tight layouts |
//! | [`RegularProductViewStyle`] | Style: Standard platform-appropriate product layout |
//! | [`LargeProductViewStyle`] | Style: Large hero product view layout |
//! | [`IconPhaseStoreView`] | Initializer: Store collection with icon phases |
//! | [`PlaceholderIconStoreView`] | Initializer: Store collection with placeholder icons |
//! | [`CustomIconStoreView`] | Initializer: Store collection with custom icons |
//! | [`StoreViewElement`] | Initializer: Loads and merchandises a product collection |
//! | [`StoreProduct`] | Type: Purchasable product/option data shared by store views |
//! | [`StoreCancellationButton`] | Modifier: Dismisses the current store presentation |
//! | [`RestorePurchasesButton`] | Modifier: Restores previously purchased products |
//! | [`CustomGroupSubscriptionStoreView`] | Initializer: Subscription store with custom grouping |
//! | [`CustomHeaderSubscriptionStoreView`] | Initializer: Subscription store with custom header |
//! | [`UpgradeOnlySubscriptionStoreView`] | Initializer: Subscription store showing upgrades only |
//! | [`GroupSubscriptionStoreView`] | Initializer: Loads all subscriptions in a group |
//! | [`SingleSubscriptionStoreView`] | Initializer: Loads subscriptions for a single product |
//! | [`SubscriptionStoreViewElement`] | Initializer: Loads subscriptions for a product collection |
//! | [`CapsuleTextField`] | Modifier: Gives your text field a capsule shape (macOS Liquid Glass) |

pub mod pickers;
pub mod text_input;
// Shims kept for backwards compatibility — real code lives in `pickers/`
pub mod wheel_picker;
pub mod picker;
pub mod progress_view;
pub mod sidebar;
pub mod content_unavailable_view;
pub mod elements;

pub use text_input::TextInput;
pub use pickers::wheel_picker::WheelPicker;
pub use pickers::picker::{Picker, PickerItem, PickerSection, PickerStyle};
pub use pickers::card::PickerCard;
pub use progress_view::{ProgressView, ProgressViewStyle};
pub use sidebar::Sidebar;
pub use content_unavailable_view::ContentUnavailableView;
pub use elements::buttons::{Button, ButtonRole, ButtonStyle, ButtonBorderShape, ButtonSizing, RenameButton, EditButton, PasteButton};
pub use elements::colors::{
    ColorGradient, ColorOpacity, ColorVariants, HierarchicalVariant, SemanticColor,
    SemanticColors, StandardColor, StandardColors, UIKitBackgroundColor,
    UIKitContentBackgroundColors, UIKitFillColor, UIKitFillColors, UIKitLabelColor,
    UIKitLabelColors, UIKitSeparatorColor, UIKitSeparatorColors, UIKitTextColor,
    UIKitTextColors,
};
pub use elements::control_groups::{
    CompactMenuControlGroupStyle, ControlGroup, ControlGroupStyle, MenuControlGroupStyle,
    NavigationControlGroupStyle, PaletteControlGroupStyle,
};
pub use elements::concentric_rectangles::{ConcentricRectangle, UniformConcentricRectangle};
pub use elements::async_images::{
    AsyncImagePhase, AsyncURLImage, CustomPhasesAsyncImage, CustomPlaceholderAsyncImage,
    CustomSessionAsyncImage,
};
pub use elements::product_views::{
    CompactProductViewStyle, CustomIconProductView, LargeProductViewStyle,
    PlaceholderIconProductView, ProductViewElement, RegularProductViewStyle,
};
pub use elements::store_views::{
    CustomIconStoreView, IconPhaseStoreView, PlaceholderIconStoreView, RestorePurchasesButton,
    StoreCancellationButton, StoreIconPhase, StoreViewElement,
};
pub use elements::subscription_store_views::{
    CustomGroupSubscriptionStoreView, CustomHeaderSubscriptionStoreView, GroupSubscriptionStoreView,
    SingleSubscriptionStoreView, SubscriptionStoreViewElement, UpgradeOnlySubscriptionStoreView,
};
pub use elements::text_fields::CapsuleTextField;
pub use elements::links::{CustomPreviewShareLink, HelpLink, Link, ShareLink, TextFieldLink};
pub use elements::labeled_contents::{CustomLabeledContent, FormattedLabeledContent, LabeledContent};
pub use elements::labels::{CustomLabel, ImageLabel, LabelStyleKind, LabelStyles, SystemImageLabel};
pub use elements::materials::{Material, Materials};
pub use elements::navigation::NavigationSubtitle;
pub use elements::tab_views::{
    BottomAccessory, DefaultAdaptableTabBarPlacement, DefaultCollapsedTabSection,
    GroupedTabViewStyle, HiddenIndexPageTabViewStyle, HideTabBarOnScrollDown, PageTabViewStyle,
    SearchTabRole, SidebarAdaptableTabViewStyle, TabBadge, TabBarOnlyTabViewStyle,
    TabBarSectionActions, TabSection, TabView, TabViewBottomAccessoryPlacement,
    TabViewCustomization, TabViewCustomizationBehavior, TabViewSideBarBottomBar,
    TabViewSideBarFooter, TabViewSideBarHeader, ValueTabView, VerticalPageTabViewStyle,
};
pub use elements::text::{TextFormat, TextFormatKind};
pub use elements::views::{
    AppStoreOverlay, BackgroundExtensionEffect, ControlSizeView, GlassEffect,
    ManageSubscriptionsSheet, MusicPicker, NavigationContainerBackground,
    NavigationSplitViewBackground, SwipeAction, SwipeContainer,
};
pub use elements::dividers::{Divider, DividerOrientation};
pub use elements::group_boxes::GroupBox;
pub use elements::lists::{List, ListSection, ListRow, ListStyle, OutlineGroup, DisclosureGroup};
pub use elements::scroll_views::{ScrollView, ScrollEdgeEffect};
pub use elements::shapes::{
    Capsule, Circle, ContainerRelativeShape, Ellipse, RectangleShape,
    RoundedRectangle, UnevenRoundedRectangle,
};
pub use elements::sliders::Slider;
pub use elements::store_product::StoreProduct;
pub use elements::sheets::{
    BooleanSheet, DisableSheetDismissSwipe, FittedSheetSizing, ItemSheet,
    PageScreenSheetSize, PrioritizeSheetContentScrolling, SheetBackground,
    SheetBackgroundInteraction, SheetBackgroundInteractionKind, SheetCornerRadius,
    SheetDetent, SheetDragIndicator, SheetDragIndicatorVisibility, SheetPlacement,
    SheetPlacementKind, SheetSize,
};
pub use elements::gauges::{Gauge, GaugeStyle, GaugeTint};
pub use elements::menus::{Menu, MenuItem, MenuEntry, MenuRole, ContextMenu};
pub use elements::toggles::{Toggle, ToggleStyle};
pub use elements::toolbars::{Toolbar, ToolbarItem, ToolbarSpacer, ToolbarItemPlacement, ToolbarSpacerSizing};
pub use elements::view_that_fits::{ViewThatFits, ViewThatFitsAxis};
#[cfg(feature = "coreicon")]
pub use sidebar::SidebarIcon;

pub const TONTOO_UI_VERSION: (u32, u32, u32) = (0, 1, 0);

pub mod prelude {
    pub use crate::{
        AppStoreOverlay, BackgroundExtensionEffect, BottomAccessory, ColorGradient, ColorOpacity,
        ColorVariants, CompactMenuControlGroupStyle, ControlGroup, ControlGroupStyle,
        ControlSizeView, CustomPreviewShareLink, DefaultAdaptableTabBarPlacement,
        DefaultCollapsedTabSection, GlassEffect, GroupedTabViewStyle, HelpLink,
        HiddenIndexPageTabViewStyle, HideTabBarOnScrollDown, HierarchicalVariant, Link, Material,
        Materials, MenuControlGroupStyle, MusicPicker, NavigationContainerBackground,
        NavigationSplitViewBackground, NavigationSubtitle, PageTabViewStyle, PaletteControlGroupStyle,
        SearchTabRole, SemanticColor, SemanticColors, ShareLink, SidebarAdaptableTabViewStyle,
        StandardColor, StandardColors, SwipeAction, SwipeContainer, TabBadge,
        TabBarOnlyTabViewStyle, TabBarSectionActions, TabSection, TabView,
        TabViewBottomAccessoryPlacement, TabViewCustomization, TabViewCustomizationBehavior,
        TabViewSideBarBottomBar, TabViewSideBarFooter, TabViewSideBarHeader, TextFieldLink,
        TextFormat, TextFormatKind, UIKitBackgroundColor, UIKitContentBackgroundColors,
        UIKitFillColor, UIKitFillColors, UIKitLabelColor, UIKitLabelColors, UIKitSeparatorColor,
        UIKitSeparatorColors, UIKitTextColor, UIKitTextColors, ValueTabView,
        VerticalPageTabViewStyle, TextInput, WheelPicker, Picker, PickerItem, PickerSection,
        PickerStyle, PickerCard, ProgressView, ProgressViewStyle, Sidebar, ContentUnavailableView,
        TONTOO_UI_VERSION,
    };
    pub use crate::elements::dividers::{Divider, DividerOrientation};
    pub use crate::elements::async_images::{
        AsyncImagePhase, AsyncURLImage, CustomPhasesAsyncImage, CustomPlaceholderAsyncImage,
        CustomSessionAsyncImage,
    };
    pub use crate::elements::product_views::{
        CompactProductViewStyle, CustomIconProductView, LargeProductViewStyle,
        PlaceholderIconProductView, ProductViewElement, RegularProductViewStyle,
    };
    pub use crate::elements::store_views::{
        CustomIconStoreView, IconPhaseStoreView, PlaceholderIconStoreView, RestorePurchasesButton,
        StoreCancellationButton, StoreIconPhase, StoreViewElement,
    };
    pub use crate::elements::subscription_store_views::{
        CustomGroupSubscriptionStoreView, CustomHeaderSubscriptionStoreView,
        GroupSubscriptionStoreView, SingleSubscriptionStoreView, SubscriptionStoreViewElement,
        UpgradeOnlySubscriptionStoreView,
    };
    pub use crate::elements::text_fields::CapsuleTextField;
    pub use crate::elements::concentric_rectangles::{ConcentricRectangle, UniformConcentricRectangle};
    pub use crate::elements::labeled_contents::{CustomLabeledContent, FormattedLabeledContent, LabeledContent};
    pub use crate::elements::labels::{CustomLabel, ImageLabel, LabelStyleKind, LabelStyles, SystemImageLabel};
    pub use crate::elements::group_boxes::GroupBox;
    pub use crate::elements::lists::{List, ListSection, ListRow, ListStyle, OutlineGroup, DisclosureGroup};
    pub use crate::elements::sliders::Slider;
    pub use crate::elements::store_product::StoreProduct;
    pub use crate::elements::shapes::{
        Capsule, Circle, ContainerRelativeShape, Ellipse, RectangleShape,
        RoundedRectangle, UnevenRoundedRectangle,
    };
    pub use crate::elements::sheets::{
        BooleanSheet, DisableSheetDismissSwipe, FittedSheetSizing, ItemSheet,
        PageScreenSheetSize, PrioritizeSheetContentScrolling, SheetBackground,
        SheetBackgroundInteraction, SheetBackgroundInteractionKind, SheetCornerRadius,
        SheetDetent, SheetDragIndicator, SheetDragIndicatorVisibility, SheetPlacement,
        SheetPlacementKind, SheetSize,
    };
    pub use crate::elements::gauges::{Gauge, GaugeStyle, GaugeTint};
    pub use crate::elements::menus::{Menu, MenuItem, MenuEntry, MenuRole, ContextMenu};
    pub use crate::elements::view_that_fits::{ViewThatFits, ViewThatFitsAxis};
    pub use crate::elements::buttons::{Button, ButtonRole, ButtonStyle, ButtonBorderShape, ButtonSizing, RenameButton, EditButton, PasteButton};
    pub use crate::elements::toggles::{Toggle, ToggleStyle};
    pub use crate::elements::toolbars::{Toolbar, ToolbarItem, ToolbarSpacer, ToolbarItemPlacement, ToolbarSpacerSizing};
    #[cfg(feature = "coreicon")]
    pub use crate::SidebarIcon;

    // Re-export UIKit types for convenience
    pub use uikit::prelude::*;
}

mod ffi;
