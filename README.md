# Tontoo TontooUI

A Swift like UI Lib based on UIKit for making Modern Apps

## Made for TontooOS

Explore more at https://github.com/TontooOS/Libs

## Adding to Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
sdk = { path = "/Library/System/sdk", features = ["TontooUI"] }
```

Then at the crate root:

```rust
sdk::preinclude!();
use TontooUI::{ /* ... */ };
```

## License

TCL v26.1