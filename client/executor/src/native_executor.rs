// This file is part of europa which is forked form Substrate.

// Copyright (C) 2017-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// Copyright 2020-2021 patract labs. Licensed under GPL-3.0.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::{
	panic::{AssertUnwindSafe, UnwindSafe},
	result,
};

use codec::{Decode, Encode};
use sc_executor::{
	error::{Error, Result},
	with_externalities_safe, NativeExecutionDispatch, RuntimeInfo,
};
use sp_core::{
	traits::{CodeExecutor, Externalities, RuntimeCode},
	NativeOrEncoded,
};
use sp_version::{NativeVersion, RuntimeVersion};

/// A generic `CodeExecutor` implementation that uses a delegate to determine wasm code equivalence
/// and dispatch to native code when possible, falling back on `WasmExecutor` when not.
pub struct NativeExecutor<D> {
	/// Dummy field to avoid the compiler complaining about us not using `D`.
	_dummy: std::marker::PhantomData<D>,
	/// Native runtime version info.
	native_version: NativeVersion,
}

impl<D: NativeExecutionDispatch> NativeExecutor<D> {
	/// Create new instance.
	pub fn new() -> Self {
		NativeExecutor {
			_dummy: Default::default(),
			native_version: D::native_version(),
		}
	}
}

impl<D: NativeExecutionDispatch> RuntimeInfo for NativeExecutor<D> {
	fn native_version(&self) -> &NativeVersion {
		&self.native_version
	}

	fn runtime_version(
		&self,
		_ext: &mut dyn Externalities,
		_runtime_code: &RuntimeCode,
	) -> Result<RuntimeVersion> {
		// do not use wasm runtime version, use native runtime version directly
		Ok(self.native_version.runtime_version.clone())
	}
}

impl<D: NativeExecutionDispatch + 'static> CodeExecutor for NativeExecutor<D> {
	type Error = Error;

	fn call<
		R: Decode + Encode + PartialEq,
		NC: FnOnce() -> result::Result<R, Box<dyn std::error::Error + Send + Sync>> + UnwindSafe,
	>(
		&self,
		ext: &mut dyn Externalities,
		_runtime_code: &RuntimeCode,
		method: &str,
		data: &[u8],
		_use_native: bool,
		native_call: Option<NC>,
	) -> (Result<NativeOrEncoded<R>>, bool) {
		let mut ext = AssertUnwindSafe(ext);
		let result = if let Some(call) = native_call {
			with_externalities_safe(&mut **ext, move || (call)())
				.and_then(|r| r.map(NativeOrEncoded::Native).map_err(Error::ApiError))
		} else {
			D::dispatch(&mut **ext, method, data).map(NativeOrEncoded::Encoded)
		};

		(result, true)
	}
}

impl<D: NativeExecutionDispatch> Clone for NativeExecutor<D> {
	fn clone(&self) -> Self {
		NativeExecutor {
			_dummy: Default::default(),
			native_version: D::native_version(),
		}
	}
}

impl<D: NativeExecutionDispatch> sp_core::traits::ReadRuntimeVersion for NativeExecutor<D> {
	fn read_runtime_version(
		&self,
		_: &[u8],
		_: &mut dyn Externalities,
	) -> std::result::Result<Vec<u8>, String> {
		unimplemented!("Not required in Europa.")
	}
}
/// Implements a `NativeExecutionDispatch` for provided parameters.
///
/// # Example
///
/// ```ignore
/// sc_executor::native_executor_instance!(
///     pub MyExecutor,
///     substrate_test_runtime::api::dispatch,
///     substrate_test_runtime::native_version,
/// );
/// ```
#[macro_export]
macro_rules! native_executor_instance {
	( $pub:vis $name:ident, $dispatcher:path, $version:path $(,)?) => {
		/// A unit struct which implements `NativeExecutionDispatch` feeding in the
		/// hard-coded runtime.
		$pub struct $name;
		$crate::native_executor_instance!(IMPL $name, $dispatcher, $version);
	};
	(IMPL $name:ident, $dispatcher:path, $version:path) => {
		impl $crate::NativeExecutionDispatch for $name {
			type ExtendHostFunctions = ();

			fn dispatch(
				ext: &mut dyn $crate::Externalities,
				method: &str,
				data: &[u8]
			) -> $crate::error::Result<Vec<u8>> {
				$crate::with_externalities_safe(ext, move || $dispatcher(method, data))?
					.ok_or_else(|| $crate::error::Error::MethodNotFound(method.to_owned()))
			}

			fn native_version() -> $crate::NativeVersion {
				$version()
			}
		}
	}
}
