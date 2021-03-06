/*!
Your adventure start with a choice:

Do you wish to inspect 64-bit PE binares? ⟶ [continue](pe64/index.html)

Do you wish to inspect 32-bit PE binaries? ⟶ [continue](pe32/index.html)

The `pelite::pe` module is aliased to the target of the compiled crate.
Use it if you want to work with modules in your own process.
Evidently only available on Windows targets.

Due to small but incompatible differences the two formats are not unified.
*/

#![recursion_limit = "128"]

pub mod image;

#[macro_use]
mod strings;

#[macro_use]
pub mod util;

pub mod pattern;

mod error;
pub use self::error::{Error, Result};

mod mmap;
#[cfg(windows)]
pub use self::mmap::{FileMap, ImageMap};
#[cfg(unix)]
pub use self::mmap::{FileMap};

pub mod pe64;
pub mod pe32;

/// Defaults to the current platform if it is available.
#[cfg(all(windows, target_pointer_width = "32"))]
pub use pe32 as pe;
/// Defaults to the current platform if it is available.
#[cfg(all(windows, target_pointer_width = "64"))]
pub use pe64 as pe;

pub mod resources;
