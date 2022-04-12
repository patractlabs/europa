// This file is part of europa

// Copyright 2020-2022 Patract Labs. Licensed under GPL-3.0.

//! Util
use crate::{
	imp::{Trap as OutterTrap, TrapCode as OutterTrapCode},
	Error, HostFuncType, ReturnValue, Value,
};
use sp_std::{fmt, mem};
use wasmtime::{
	Caller,
	Config,
	Engine,
	Func,
	FuncType,
	Store,
	Trap,
	TrapCode,
	Val, // WasmBacktraceDetails,
};

/// Create store with DWARF enabled
///
/// NOTE: The Debug info with native trace has some problem in
/// aarch64-apple-darwin, not enable for default for now.
pub fn store_with_dwarf() -> Store {
	let config = Config::new();
	// .debug_info(true)
	// .wasm_backtrace_details(WasmBacktraceDetails::Enable),
	Store::new(&Engine::new(&config).expect("init wasmtime engine fail"))
}

/// Wrap host function into `Func`
pub fn wrap_fn<T>(store: &Store, state: usize, f: usize, sig: FuncType) -> Func {
	let func = move |_: Caller<'_>, args: &[Val], results: &mut [Val]| {
		let mut inner_args = vec![];
		for arg in args {
			if let Some(arg) = from_val(arg.clone()) {
				inner_args.push(arg);
			} else {
				return Err(Trap::new("Could not wrap host function"));
			}
		}

		// HACK the LIFETIME
		//
		// # Safety
		//
		// Work for one call.
		let state: &mut T = unsafe { mem::transmute(state) };
		let func: HostFuncType<T> = unsafe { mem::transmute(f) };
		match func(state, &inner_args) {
			Ok(ret) => {
				if let Some(ret) = from_ret_val(ret) {
					results[0] = ret;
				}
				Ok(())
			}
			Err(e) => Err(Trap::new(format!("{:?}", e))),
		}
	};
	Func::new(store, sig, func)
}

pub fn from_val(v: Val) -> Option<Value> {
	match v {
		Val::F32(v) => Some(Value::F32(v)),
		Val::I32(v) => Some(Value::I32(v)),
		Val::F64(v) => Some(Value::F64(v)),
		Val::I64(v) => Some(Value::I64(v)),
		_ => None,
	}
}

pub fn to_val(v: Value) -> Val {
	match v {
		Value::F32(v) => Val::F32(v),
		Value::F64(v) => Val::F64(v),
		Value::I32(v) => Val::I32(v),
		Value::I64(v) => Val::I64(v),
	}
}

pub fn to_ret_val(v: Val) -> Option<ReturnValue> {
	from_val(v).map(|v| ReturnValue::Value(v))
}

fn from_ret_val(v: ReturnValue) -> Option<Val> {
	match v {
		ReturnValue::Value(v) => match v {
			Value::I64(v) => Some(Val::I64(v)),
			Value::F64(v) => Some(Val::F64(v)),
			Value::I32(v) => Some(Val::I32(v)),
			Value::F32(v) => Some(Val::F32(v)),
		},
		ReturnValue::Unit => None,
	}
}

impl From<Trap> for Error {
	fn from(trap: Trap) -> Error {
		let mut code = OutterTrapCode::Unknown;
		if let Some(cc) = trap.trap_code() {
			code = match cc {
				TrapCode::BadConversionToInteger => OutterTrapCode::BadConversionToInteger,
				TrapCode::BadSignature => OutterTrapCode::BadSignature,
				TrapCode::HeapMisaligned => OutterTrapCode::HeapMisaligned,
				TrapCode::IndirectCallToNull => OutterTrapCode::IndirectCallToNull,
				TrapCode::IntegerDivisionByZero => OutterTrapCode::IntegerDivisionByZero,
				TrapCode::IntegerOverflow => OutterTrapCode::IntegerOverflow,
				TrapCode::Interrupt => OutterTrapCode::Interrupt,
				TrapCode::MemoryOutOfBounds => OutterTrapCode::MemoryOutOfBounds,
				TrapCode::StackOverflow => OutterTrapCode::StackOverflow,
				TrapCode::TableOutOfBounds => OutterTrapCode::TableOutOfBounds,
				TrapCode::UnreachableCodeReached => OutterTrapCode::UnreachableCodeReached,
				_ => OutterTrapCode::Unknown,
			}
		}

		Error::Trap(OutterTrap {
			code,
			trace: format!("{}", trap)
				.split("\n")
				.map(|s| s.to_string())
				.collect::<Vec<_>>(),
		})
	}
}

impl fmt::Display for OutterTrap {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		self.trace
			.iter()
			.map(|s| write!(f, "{}", s))
			.collect::<Result<Vec<_>, fmt::Error>>()?;
		Ok(())
	}
}
