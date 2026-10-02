1. Glass effekt with blur should work through the app if the window cant move via making a screenshot then showing it behind the app like there would be no screenshot

   Done: `CompositorBackdrop` in `src/renderer/backdrop_stream.rs`. The
   compositor captures the elements below the window into a shared memory
   file at half resolution, TontooUI upsamples it and blurs it with the
   existing WGSL pass, so the glass reads as if the desktop were directly
   behind it. See `wiki/BackdropStream.md`. Compositors without the
   `tontoo_ui_manager` global keep the old in-app capture pass.
