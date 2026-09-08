//! StoreProduct — purchasable product or subscription option data.
//!
//! Shared data model for the `product_views`, `store_views` and
//! `subscription_store_views` categories. Views render from this data and
//! report interaction through callbacks (`on_buy`, `on_select`,
//! `on_subscribe`, ...). There is no real store backend — the hosting app
//! connects its own kit (e.g. StoreKit) inside the callbacks.

/// One purchasable product or subscription option.
///
/// All product/store/subscription views ship with gallery defaults
/// (`VIP Kitty Pass` / `Kitty Hat`) so existing code keeps compiling;
/// hosts replace them through the builders.
#[derive(Debug, Clone, PartialEq)]
pub struct StoreProduct {
    /// Display title (e.g. `"VIP Kitty Pass"`).
    pub title: String,
    /// Subtitle line (e.g. `"Enjoy the full premium kitty experience"`).
    pub subtitle: String,
    /// Price line (e.g. `"$0.99/month"`).
    pub price: String,
}

impl StoreProduct {
    /// Create a product with title, subtitle and price.
    pub fn new(
        title: impl Into<String>,
        subtitle: impl Into<String>,
        price: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            subtitle: subtitle.into(),
            price: price.into(),
        }
    }

    /// The gallery default product (monthly pass).
    pub fn monthly_default() -> Self {
        Self::new(
            "VIP Kitty Pass",
            "Enjoy the full premium kitty experience",
            "$0.99/month",
        )
    }

    /// The gallery default second product (one-time hat).
    pub fn hat_default() -> Self {
        Self::new("Kitty Hat", "A cute hat for your kitty", "$1.99")
    }

    /// The gallery default yearly option.
    pub fn yearly_default() -> Self {
        Self::new(
            "VIP Kitty Pass",
            "1 week free, then $9.99/year",
            "$9.99/year",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_exist() {
        assert_eq!(StoreProduct::monthly_default().price, "$0.99/month");
        assert_eq!(StoreProduct::hat_default().title, "Kitty Hat");
        assert_eq!(StoreProduct::yearly_default().price, "$9.99/year");
    }
}
