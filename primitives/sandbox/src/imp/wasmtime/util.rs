//! Util
use crate::{HostFuncType, ReturnValue, Value};
use sp_std::mem;
use wasmtime::{Caller, Func, FuncType, Store, Trap, Val};

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
