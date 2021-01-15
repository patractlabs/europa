//! Util
use crate::{FunctionType, HostFuncType, ReturnValue, Value};
use parity_wasm::elements::ValueType;
use sp_std::mem;
use wasmtime::{Caller, Func, FuncType, Store, Trap, Val, ValType};

pub fn to_val_ty(ty: ValueType) -> ValType {
	match ty {
		ValueType::I32 => ValType::I32,
		ValueType::F32 => ValType::F32,
		ValueType::F64 => ValType::F64,
		ValueType::I64 => ValType::I64,
	}
}

fn wasmtime_sig(sig: FunctionType) -> FuncType {
	let params = sig
		.params()
		.iter()
		.map(|ty| to_val_ty(*ty))
		.collect::<Vec<_>>();
	let results = if let Some(ret) = sig.return_type().map(to_val_ty) {
		vec![ret]
	} else {
		vec![]
	};

	FuncType::new(params, results)
}

pub fn wrap_fn<T>(store: &Store, state: usize, f: usize, sig: FunctionType) -> Func {
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
		// Runtime only run for one call.
		let state: &mut T = unsafe { mem::transmute(state) };
		let func: HostFuncType<T> = unsafe { mem::transmute(f) };
		match func(state, &inner_args) {
			Ok(ret) => {
				if let Some(ret) = from_ret_val(ret) {
					// TODO: check the signature
					results[0] = ret;
				}
				Ok(())
			}
			Err(_) => Err(Trap::new("Could not wrap host function")),
		}
	};
	Func::new(store, wasmtime_sig(sig), func)
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
