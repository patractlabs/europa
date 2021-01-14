//! Util
use crate::{FunctionType, HostFuncType, ReturnValue, Value};
use parity_wasm::elements::ValueType;
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

fn wrap_fn<T>(store: &Store, state: &mut T, f: HostFuncType<T>, sig: FunctionType) -> Func {
	let state_mut = state as *mut T;
	let func = move |_: Caller<'_>, args: &[Val], results: &mut [Val]| {
		let result = unsafe { f(*state_mut, args) };
		match result {
			Ok(ret) => {
				if let Some(ret) = from_ret_val(ret) {
					results = &[ret];
				}
				Ok(())
			}
			Err(_) => Err(Trap::new("Could not wrap host function")),
		}
		Ok(())
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

pub fn from_ret_val(v: ReturnValue) -> Option<Val> {
	match v {
		ReturnValue::Value(v) => match v {
			Value::I64(v) => Some(Val::I64(v)),
			Value::F64(v) => Some(Val::F64(v)),
			Value::I32(v) => Some(Val::I32(v)),
			Value::F32(v) => Some(Val::F32(v)),
			_ => None,
		},
		ReturnValue::Unit => None,
	}
}
