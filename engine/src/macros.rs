/// Tell vg about your game
/// 
/// # Usage
/// ```rust
/// game!(my_game);
/// 
/// async fn my_game(vg: Vg) {
///     // ...
/// }
/// ```
#[macro_export]
macro_rules! game {
    ($i:ident) => {
        /// VG INTERNAL GAME HANDLE
        #[allow(improper_ctypes)]
        pub extern "C" fn __vg_interal_game_handle(
            vg: Vg,
        ) -> Box<Box<dyn core::future::Future<Output = ()>>> {
            Box::new(Box::new($i(vg)))
        }
    };
}
