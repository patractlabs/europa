// This file is part of europa

// Copyright 2020-2021 patract labs. Licensed under GPL-3.0.

#[macro_use]
mod native_executor;

pub use native_executor::NativeExecutor;
pub use sc_executor::{with_externalities_safe, NativeExecutionDispatch, RuntimeInfo};
pub use sc_executor_common::error;
pub use sp_core::traits::Externalities;
pub use sp_version::{NativeVersion, RuntimeVersion};
