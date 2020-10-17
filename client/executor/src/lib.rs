#[macro_use]
mod native_executor;

pub use native_executor::{with_externalities_safe, NativeExecutor, NativeVersion};
pub use sc_executor::NativeExecutionDispatch;
pub use sp_core::Externalities;
