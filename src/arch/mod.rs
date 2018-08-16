#[cfg(target_os = "android")]
#[macro_use]
mod android;
#[cfg(not(all(target_os = "android")))]
#[macro_use]
mod desktop;

#[cfg(target_os = "android")]
pub use self::android::*;
#[cfg(not(all(target_os = "android")))]
pub use self::desktop::*;
