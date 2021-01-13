#[cfg(feature = "interpreter")]
mod wasmi;
#[cfg(feature = "interpreter")]
pub use self::wasmi::*;

#[cfg(feature = "jit")]
mod wasmtime;
#[cfg(feature = "jit")]
pub use self::wasmtime::*;
