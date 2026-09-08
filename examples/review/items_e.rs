//! Review items: Sheet (12), Shapes (7), Label (4), LabeledContent (3),
//! ConcentricRectangle (2), AsyncImage (4), ProductView (6), StoreView (6),
//! SubscriptionStoreView (6), TextField (1), Divider (4), Gauge (11), Sidebar (1).

use super::ReviewItem;
use tontooui::prelude::*;

pub fn items(out: &mut Vec<ReviewItem>) {
    // ── Sheet (12) ──
    out.push(ReviewItem { id: "sheet_placement", category: "Sheet", title: "Sheet Placement", desc: "Sets the placement of a presentation within the presenting view.", badge: "modifier", make: || WidgetNode::new(SheetPlacement::new()) });
    out.push(ReviewItem { id: "sheet_no_dismiss", category: "Sheet", title: "Disable Sheet Dismiss Swipe", desc: "Conditionally prevents interactive dismissal of presentations.", badge: "modifier", make: || WidgetNode::new(DisableSheetDismissSwipe::new()) });
    out.push(ReviewItem { id: "sheet_page_size", category: "Sheet", title: "Page Screen Sheet Size", desc: "On devices smaller than a page of paper, page sizing fills.", badge: "modifier", make: || WidgetNode::new(PageScreenSheetSize::new()) });
    out.push(ReviewItem { id: "sheet_fitted", category: "Sheet", title: "Fitted Sheet Sizing", desc: "Sets the sizing of the containing presentation.", badge: "modifier", make: || WidgetNode::new(FittedSheetSizing::new()) });
    out.push(ReviewItem { id: "sheet_corner", category: "Sheet", title: "Sheet Corner Radius", desc: "Requests that the presentation have a specific corner radius.", badge: "modifier", make: || WidgetNode::new(SheetCornerRadius::new()) });
    out.push(ReviewItem { id: "sheet_prioritize", category: "Sheet", title: "Prioritize Sheet Content Scrolling", desc: "Configure the behavior of swipe gestures on a presentation.", badge: "modifier", make: || WidgetNode::new(PrioritizeSheetContentScrolling::new()) });
    out.push(ReviewItem { id: "sheet_bg_interaction", category: "Sheet", title: "Sheet Background Interaction", desc: "Controls whether people can interact with the view behind a presentation.", badge: "modifier", make: || WidgetNode::new(SheetBackgroundInteraction::new()) });
    out.push(ReviewItem { id: "sheet_bg", category: "Sheet", title: "Sheet Background", desc: "Sets the presentation background of the enclosing sheet.", badge: "modifier", make: || WidgetNode::new(SheetBackground::new()) });
    out.push(ReviewItem { id: "sheet_drag", category: "Sheet", title: "Sheet Drag Indicator Visibility", desc: "Sets the visibility of the drag indicator on top of a sheet.", badge: "modifier", make: || WidgetNode::new(SheetDragIndicatorVisibility::new()) });
    out.push(ReviewItem { id: "sheet_size", category: "Sheet", title: "Sheet Size", desc: "Sets the available detents for the enclosing sheet.", badge: "modifier", make: || WidgetNode::new(SheetSize::new()) });
    out.push(ReviewItem { id: "sheet_item", category: "Sheet", title: "Item Sheet", desc: "Presents a sheet using the given item as a data source.", badge: "modifier", make: || WidgetNode::new(ItemSheet::new()) });
    out.push(ReviewItem { id: "sheet_boolean", category: "Sheet", title: "Boolean Sheet", desc: "Presents a sheet when a binding to a Boolean value is true.", badge: "modifier", make: || WidgetNode::new(BooleanSheet::new()) });

    // ── Shapes (7) ──
    out.push(ReviewItem { id: "shape_circle", category: "Shapes", title: "Circle", desc: "A circle shape centered in its frame.", badge: "modifier", make: || WidgetNode::new(Circle::new()) });
    out.push(ReviewItem { id: "shape_ellipse", category: "Shapes", title: "Ellipse", desc: "An elliptical shape filling its frame.", badge: "modifier", make: || WidgetNode::new(Ellipse::new()) });
    out.push(ReviewItem { id: "shape_capsule", category: "Shapes", title: "Capsule", desc: "A capsule (stadium) shape filling its frame.", badge: "modifier", make: || WidgetNode::new(Capsule::new()) });
    out.push(ReviewItem { id: "shape_rect", category: "Shapes", title: "Rectangle", desc: "A rectangular shape filling its frame.", badge: "modifier", make: || WidgetNode::new(RectangleShape::new()) });
    out.push(ReviewItem { id: "shape_rrect", category: "Shapes", title: "Rounded Rectangle", desc: "A rectangle with rounded corners.", badge: "modifier", make: || WidgetNode::new(RoundedRectangle::new()) });
    out.push(ReviewItem { id: "shape_urect", category: "Shapes", title: "Uneven Rounded Rectangle", desc: "A rectangle with uneven corner radii.", badge: "modifier", make: || WidgetNode::new(UnevenRoundedRectangle::new()) });
    out.push(ReviewItem { id: "shape_crs", category: "Shapes", title: "ContainerRelativeShape", desc: "A shape that is replaced by an inset version of the current container shape.", badge: "modifier", make: || WidgetNode::new(ContainerRelativeShape::new()) });

    // ── Label (4) ──
    out.push(ReviewItem { id: "label_custom", category: "Label", title: "Custom Label", desc: "Creates a label with a custom title and icon.", badge: "initializer", make: || WidgetNode::new(CustomLabel::new()) });
    out.push(ReviewItem { id: "label_image", category: "Label", title: "Image Label", desc: "Creates a label with an icon image and a localized title.", badge: "initializer", make: || WidgetNode::new(ImageLabel::new()) });
    out.push(ReviewItem { id: "label_system", category: "Label", title: "System Image Label", desc: "Creates a label with a system icon and a localized title.", badge: "initializer", make: || WidgetNode::new(SystemImageLabel::new()) });
    out.push(ReviewItem { id: "label_styles", category: "Label", title: "Label Styles", desc: "Sets the style for labels within this view.", badge: "style", make: || WidgetNode::new(LabelStyles::new()) });

    // ── LabeledContent (3) ──
    out.push(ReviewItem { id: "lc_custom", category: "LabeledContent", title: "Custom Labeled Content", desc: "Creates a standard labeled element with a custom value view.", badge: "initializer", make: || WidgetNode::new(CustomLabeledContent::new()) });
    out.push(ReviewItem { id: "lc_formatted", category: "LabeledContent", title: "Formatted Labeled Content", desc: "Creates a labeled informational view from a formatted value.", badge: "initializer", make: || WidgetNode::new(FormattedLabeledContent::new()) });
    out.push(ReviewItem { id: "lc_plain", category: "LabeledContent", title: "Labeled Content", desc: "Creates a labeled informational view.", badge: "initializer", make: || WidgetNode::new(LabeledContent::new()) });

    // ── ConcentricRectangle (2) ──
    out.push(ReviewItem { id: "cr_uniform", category: "ConcentricRectangle", title: "Uniform Concentric Rectangle", desc: "Create a rectangle with the same corner style set on four corners.", badge: "initializer", make: || WidgetNode::new(UniformConcentricRectangle::new()) });
    out.push(ReviewItem { id: "cr_plain", category: "ConcentricRectangle", title: "Concentric Rectangle", desc: "A concentric rectangle whose corner radii are defined from the same circle.", badge: "initializer", make: || WidgetNode::new(ConcentricRectangle::new()) });

    // ── AsyncImage (4) ──
    out.push(ReviewItem { id: "ai_placeholder", category: "AsyncImage", title: "Custom Placeholder Async URL Image", desc: "Loads a modifiable image from a URL request with a custom placeholder.", badge: "initializer", make: || WidgetNode::new(CustomPlaceholderAsyncImage::new()) });
    out.push(ReviewItem { id: "ai_phases", category: "AsyncImage", title: "Custom Phases Async URL Image", desc: "Loads a modifiable image from a URL request with custom phases.", badge: "initializer", make: || WidgetNode::new(CustomPhasesAsyncImage::new()) });
    out.push(ReviewItem { id: "ai_url", category: "AsyncImage", title: "Async URL Image", desc: "Loads and displays an image from the specified URL load request.", badge: "initializer", make: || WidgetNode::new(AsyncURLImage::new()) });
    out.push(ReviewItem { id: "ai_session", category: "AsyncImage", title: "Custom Session Async URL Image", desc: "A modifier that adds a URL session for async images in the view.", badge: "modifier", make: || WidgetNode::new(CustomSessionAsyncImage::new()) });

    // ── ProductView (6) ──
    out.push(ReviewItem { id: "pv_placeholder", category: "ProductView", title: "Placeholder Icon Product View", desc: "Creates a view to load an individual product with a placeholder icon.", badge: "initializer", make: || WidgetNode::new(PlaceholderIconProductView::new()) });
    out.push(ReviewItem { id: "pv_custom_icon", category: "ProductView", title: "Custom Icon Product View", desc: "Creates a view to load an individual product with a custom icon.", badge: "initializer", make: || WidgetNode::new(CustomIconProductView::new()) });
    out.push(ReviewItem { id: "pv_plain", category: "ProductView", title: "Product View", desc: "Creates a view to load and merchandise an individual product.", badge: "initializer", make: || WidgetNode::new(ProductViewElement::new()) });
    out.push(ReviewItem { id: "pv_compact", category: "ProductView", title: "CompactProductViewStyle", desc: "A style for a product view for layouts with less available space.", badge: "style", make: || WidgetNode::new(CompactProductViewStyle::new()) });
    out.push(ReviewItem { id: "pv_regular", category: "ProductView", title: "RegularProductViewStyle", desc: "A style for a product view that uses a standard layout.", badge: "style", make: || WidgetNode::new(RegularProductViewStyle::new()) });
    out.push(ReviewItem { id: "pv_large", category: "ProductView", title: "LargeProductViewStyle", desc: "A style for a product view where the purchase is the hero content.", badge: "style", make: || WidgetNode::new(LargeProductViewStyle::new()) });

    // ── StoreView (6) ──
    out.push(ReviewItem { id: "sv_icon_phase", category: "StoreView", title: "Icon Phase Store View", desc: "Creates a view to load a collection of products using icon phases.", badge: "initializer", make: || WidgetNode::new(IconPhaseStoreView::new()) });
    out.push(ReviewItem { id: "sv_placeholder", category: "StoreView", title: "Placeholder Icon Store View", desc: "Creates a view to load a collection of products using placeholders.", badge: "initializer", make: || WidgetNode::new(PlaceholderIconStoreView::new()) });
    out.push(ReviewItem { id: "sv_custom_icon", category: "StoreView", title: "Custom Icon Store View", desc: "Creates a view to load a collection of products using custom icons.", badge: "initializer", make: || WidgetNode::new(CustomIconStoreView::new()) });
    out.push(ReviewItem { id: "sv_plain", category: "StoreView", title: "Store View", desc: "Creates a view to load and merchandise a collection of products.", badge: "initializer", make: || WidgetNode::new(StoreViewElement::new()) });
    out.push(ReviewItem { id: "sv_cancel", category: "StoreView", title: "Store Cancellation Button", desc: "A type of button that people use to dismiss the store presentation.", badge: "modifier", make: || WidgetNode::new(StoreCancellationButton::new()) });
    out.push(ReviewItem { id: "sv_restore", category: "StoreView", title: "Restore Purchases Button", desc: "A type of button that people use to restore purchases.", badge: "modifier", make: || WidgetNode::new(RestorePurchasesButton::new()) });

    // ── SubscriptionStoreView (6) ──
    out.push(ReviewItem { id: "ss_group", category: "SubscriptionStoreView", title: "Custom Group Subscription Store View", desc: "Creates a SubscriptionStoreView with custom grouping.", badge: "initializer", make: || WidgetNode::new(CustomGroupSubscriptionStoreView::new()) });
    out.push(ReviewItem { id: "ss_header", category: "SubscriptionStoreView", title: "Custom Header Subscription Store View", desc: "Creates a view that loads all subscriptions with a custom header.", badge: "initializer", make: || WidgetNode::new(CustomHeaderSubscriptionStoreView::new()) });
    out.push(ReviewItem { id: "ss_upgrade", category: "SubscriptionStoreView", title: "Upgrade Only Subscription Store View", desc: "Creates a view that loads upgrade options from a group.", badge: "initializer", make: || WidgetNode::new(UpgradeOnlySubscriptionStoreView::new()) });
    out.push(ReviewItem { id: "ss_group_plain", category: "SubscriptionStoreView", title: "Group Subscription Store View", desc: "Creates a view that loads all subscriptions in a group.", badge: "initializer", make: || WidgetNode::new(GroupSubscriptionStoreView::new()) });
    out.push(ReviewItem { id: "ss_single", category: "SubscriptionStoreView", title: "Single Subscription Store View", desc: "Creates a view that loads subscriptions for a single product.", badge: "initializer", make: || WidgetNode::new(SingleSubscriptionStoreView::new()) });
    out.push(ReviewItem { id: "ss_plain", category: "SubscriptionStoreView", title: "Subscription Store View", desc: "Creates a view that loads subscriptions for a collection.", badge: "initializer", make: || WidgetNode::new(SubscriptionStoreViewElement::new()) });

    // ── TextField (1) ──
    out.push(ReviewItem { id: "tf_capsule", category: "TextField", title: "Capsule Text Field", desc: "Gives your text field a capsule shape (visible on macOS).", badge: "modifier", make: || WidgetNode::new(CapsuleTextField::new()) });

    // ── Divider (4) ──
    out.push(ReviewItem { id: "div_horizontal", category: "Divider", title: "Divider Horizontal", desc: "A horizontal line that separates content.", badge: "initializer", make: || WidgetNode::new(
        VStack::new().spacing(12.0).child(Text::new("Foo").font_size(11.0)).child(Divider::horizontal().length(180.0)).child(Text::new("Bar").font_size(11.0)),
    ) });
    out.push(ReviewItem { id: "div_vertical", category: "Divider", title: "Divider Vertical", desc: "A vertical line that separates content.", badge: "initializer", make: || WidgetNode::new(
        HStack::new().spacing(12.0).child(Text::new("Left").font_size(11.0)).child(Divider::vertical().length(60.0)).child(Text::new("Right").font_size(11.0)),
    ) });
    out.push(ReviewItem { id: "div_thick", category: "Divider", title: "Divider Thick", desc: "Custom thickness and tint.", badge: "style", make: || WidgetNode::new(
        VStack::new().spacing(8.0)
            .child(Divider::horizontal().thickness(2.0).length(180.0).color(Color::from_hex("#0A84FF").unwrap()))
            .child(Divider::horizontal().thickness(1.0).length(180.0)),
    ) });
    out.push(ReviewItem { id: "div_section", category: "Divider", title: "Divider Section", desc: "Dividers inside sections.", badge: "modifier", make: || WidgetNode::new(
        VStack::new().spacing(10.0)
            .child(Divider::horizontal().length(120.0).thickness(1.0))
            .child(HStack::new().spacing(8.0)
                .child(Divider::vertical().length(40.0))
                .child(VStack::new().spacing(4.0)
                    .child(Text::new("Section").font_size(10.0).bold())
                    .child(Divider::horizontal().length(100.0))
                    .child(Text::new("Content").font_size(11.0)))),
    ) });

    // ── Gauge (11) ──
    out.push(ReviewItem { id: "gauge_minmax", category: "Gauge", title: "Min Max Current Value Gauge", desc: "Creates a gauge showing a value within a range.", badge: "initializer", make: || WidgetNode::new(
        Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").current_value_label("42").minimum_value_label("0").maximum_value_label("100").width(200.0),
    ) });
    out.push(ReviewItem { id: "gauge_current", category: "Gauge", title: "Current Value Gauge", desc: "Creates a gauge showing a value within a range.", badge: "initializer", make: || WidgetNode::new(
        Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").current_value_label("0.42000").width(200.0),
    ) });
    out.push(ReviewItem { id: "gauge_plain", category: "Gauge", title: "Gauge", desc: "Creates a gauge showing a value within a range.", badge: "initializer", make: || WidgetNode::new(
        Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").width(200.0),
    ) });
    out.push(ReviewItem { id: "gauge_colors", category: "Gauge", title: "Gauge Colors", desc: "Customizing the colors of different styles.", badge: "style", make: || WidgetNode::new(
        VStack::new().spacing(10.0)
            .child(HStack::new().spacing(18.0)
                .child(Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").current_value_label("42").gauge_style(GaugeStyle::AccessoryCircularCapacity).tint(Color::from_rgb(48, 209, 88)).width(56.0))
                .child(Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").current_value_label("42").gauge_style(GaugeStyle::AccessoryCircularCapacity).tint(Color::from_rgb(255, 159, 10)).width(56.0)))
            .child(Gauge::new(42.0).in_range(0.0, 100.0).minimum_value_label("0").maximum_value_label("100").current_value_label("42").tint(Color::from_rgb(255, 204, 0)).width(180.0)),
    ) });
    out.push(ReviewItem { id: "gauge_circular", category: "Gauge", title: "CircularGaugeStyle", desc: "A gauge style that displays an open ring with a marker.", badge: "style", make: || WidgetNode::new(
        VStack::new().spacing(12.0)
            .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").gauge_style(GaugeStyle::Circular).width(92.0))
            .child(Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").current_value_label("42").minimum_value_label("0").maximum_value_label("100").gauge_style(GaugeStyle::Circular).width(92.0)),
    ) });
    out.push(ReviewItem { id: "gauge_linear", category: "Gauge", title: "LinearGaugeStyle", desc: "A gauge style that displays a bar with a marker.", badge: "style", make: || WidgetNode::new(
        VStack::new().spacing(14.0)
            .child(Gauge::new(0.42).in_range(0.0, 1.0).gauge_style(GaugeStyle::Linear).width(180.0))
            .child(Gauge::new(42.0).in_range(0.0, 100.0).minimum_value_label("0").maximum_value_label("100").gauge_style(GaugeStyle::Linear).width(180.0)),
    ) });
    out.push(ReviewItem { id: "gauge_acc_lin_cap", category: "Gauge", title: "AccessoryLinearCapacityGaugeStyle", desc: "A gauge style that displays a bar that fills from leading to trailing.", badge: "style", make: || WidgetNode::new(
        VStack::new().spacing(14.0)
            .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").gauge_style(GaugeStyle::AccessoryLinearCapacity).width(180.0))
            .child(Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").minimum_value_label("0").maximum_value_label("100").gauge_style(GaugeStyle::AccessoryLinearCapacity).width(180.0)),
    ) });
    out.push(ReviewItem { id: "gauge_acc_lin", category: "Gauge", title: "AccessoryLinearGaugeStyle", desc: "A gauge style that displays a bar with a marker.", badge: "style", make: || WidgetNode::new(
        VStack::new().spacing(14.0)
            .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").gauge_style(GaugeStyle::AccessoryLinear).width(180.0))
            .child(Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").current_value_label("42").gauge_style(GaugeStyle::AccessoryLinear).width(180.0)),
    ) });
    out.push(ReviewItem { id: "gauge_lin_cap", category: "Gauge", title: "LinearCapacityGaugeStyle", desc: "A gauge style that displays a bar that fills from leading to trailing.", badge: "style", make: || WidgetNode::new(
        VStack::new().spacing(14.0)
            .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").gauge_style(GaugeStyle::LinearCapacity).width(180.0))
            .child(Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").minimum_value_label("0").maximum_value_label("100").gauge_style(GaugeStyle::LinearCapacity).width(180.0)),
    ) });
    out.push(ReviewItem { id: "gauge_acc_circ_cap", category: "Gauge", title: "AccessoryCircularCapacityGaugeStyle", desc: "A gauge style that displays a closed ring that is partially filled.", badge: "style", make: || WidgetNode::new(
        HStack::new().spacing(10.0)
            .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").gauge_style(GaugeStyle::AccessoryCircularCapacity).tint(Color::from_rgb(10, 132, 255)).width(56.0))
            .child(Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").current_value_label("42").gauge_style(GaugeStyle::AccessoryCircularCapacity).width(56.0)),
    ) });
    out.push(ReviewItem { id: "gauge_acc_circ", category: "Gauge", title: "AccessoryCircularGaugeStyle", desc: "A gauge style that displays an open ring with a marker.", badge: "style", make: || WidgetNode::new(
        HStack::new().spacing(10.0)
            .child(Gauge::new(0.42).in_range(0.0, 1.0).label("Foo").gauge_style(GaugeStyle::AccessoryCircular).width(56.0))
            .child(Gauge::new(42.0).in_range(0.0, 100.0).label("Foo").current_value_label("42").gauge_style(GaugeStyle::AccessoryCircular).width(56.0)),
    ) });

    // ── Sidebar (1) ──
    out.push(ReviewItem { id: "sidebar", category: "Sidebar", title: "Sidebar", desc: "macOS-style sidebar with traffic lights, search, and item list.", badge: "initializer", make: || WidgetNode::new(
        Sidebar::new().item("Foo", SidebarIcon::sf("star", Color::from_rgb(10, 132, 255))).item("Bar", SidebarIcon::sf("heart", Color::from_rgb(255, 69, 58))).selected(0),
    ) });
}
