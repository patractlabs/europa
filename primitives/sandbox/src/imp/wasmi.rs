// This file is part of europa which is forked form Substrate.

// Copyright (C) 2018-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Copyright 2020-2022 Patract Labs. Licensed under GPL-3.0.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use sp_std::{collections::btree_map::BTreeMap, fmt, mem::transmute};

use super::{Trap as OutterTrap, TrapCode};
use crate::{Error, HostError, HostFuncType, ReturnValue, Value};
use patract_wasmi::{
	memory_units::Pages, Externals, FuncInstance, FuncRef, GlobalDescriptor, GlobalRef,
	ImportResolver, MemoryDescriptor, MemoryInstance, MemoryRef, Module, ModuleInstance, ModuleRef,
	RuntimeArgs, RuntimeValue, Signature, TableDescriptor, TableRef, Trap, TrapKind,
};

#[derive(Clone)]
pub struct Memory {
	memref: MemoryRef,
}

impl Memory {
	pub fn new(initial: u32, maximum: Option<u32>) -> Result<Memory, Error> {
		Ok(Memory {
			memref: MemoryInstance::alloc(
				Pages(initial as usize),
				maximum.map(|m| Pages(m as usize)),
			)
			.map_err(|_| Error::Module)?,
		})
	}

	pub fn get(&self, ptr: u32, buf: &mut [u8]) -> Result<(), Error> {
		self.memref
			.get_into(ptr, buf)
			.map_err(|_| Error::OutOfBounds)?;
		Ok(())
	}

	pub fn set(&self, ptr: u32, value: &[u8]) -> Result<(), Error> {
		self.memref
			.set(ptr, value)
			.map_err(|_| Error::OutOfBounds)?;
		Ok(())
	}
}

struct HostFuncIndex(usize);

struct DefinedHostFunctions<T> {
	funcs: Vec<HostFuncType<T>>,
}

impl<T> Clone for DefinedHostFunctions<T> {
	fn clone(&self) -> DefinedHostFunctions<T> {
		DefinedHostFunctions {
			funcs: self.funcs.clone(),
		}
	}
}

impl<T> DefinedHostFunctions<T> {
	fn new() -> DefinedHostFunctions<T> {
		DefinedHostFunctions { funcs: Vec::new() }
	}

	fn define(&mut self, f: HostFuncType<T>) -> HostFuncIndex {
		let idx = self.funcs.len();
		self.funcs.push(f);
		HostFuncIndex(idx)
	}
}

#[derive(Debug)]
struct DummyHostError;

impl fmt::Display for DummyHostError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "DummyHostError")
	}
}

impl patract_wasmi::HostError for DummyHostError {}

struct GuestExternals<'a, T> {
	state: &'a mut T,
	defined_host_functions: &'a DefinedHostFunctions<T>,
}

impl<'a, T> Externals for GuestExternals<'a, T> {
	fn invoke_index(
		&mut self,
		index: usize,
		args: RuntimeArgs,
	) -> Result<Option<RuntimeValue>, Trap> {
		let args = args
			.as_ref()
			.iter()
			.cloned()
			.map(|v: patract_wasmi::RuntimeValue| unsafe {
				transmute::<patract_wasmi::RuntimeValue, wasmi::RuntimeValue>(v).into()
			})
			.collect::<Vec<_>>();

		let result = (self.defined_host_functions.funcs[index])(self.state, &args);
		match result {
			Ok(value) => Ok(match value {
				ReturnValue::Value(v) => {
					let wv: wasmi::RuntimeValue = v.into();
					Some(unsafe {
						transmute::<wasmi::RuntimeValue, patract_wasmi::RuntimeValue>(wv).into()
					})
				}
				ReturnValue::Unit => None,
			}),
			Err(HostError) => Err(TrapKind::Host(Box::new(DummyHostError)).into()),
		}
	}
}

enum ExternVal {
	HostFunc(HostFuncIndex),
	Memory(Memory),
}

pub struct EnvironmentDefinitionBuilder<T> {
	map: BTreeMap<(Vec<u8>, Vec<u8>), ExternVal>,
	defined_host_functions: DefinedHostFunctions<T>,
}

impl<T> EnvironmentDefinitionBuilder<T> {
	pub fn new() -> EnvironmentDefinitionBuilder<T> {
		EnvironmentDefinitionBuilder {
			map: BTreeMap::new(),
			defined_host_functions: DefinedHostFunctions::new(),
		}
	}

	pub fn add_host_func<N1, N2>(&mut self, module: N1, field: N2, f: HostFuncType<T>)
	where
		N1: Into<Vec<u8>>,
		N2: Into<Vec<u8>>,
	{
		let idx = self.defined_host_functions.define(f);
		self.map
			.insert((module.into(), field.into()), ExternVal::HostFunc(idx));
	}

	pub fn add_memory<N1, N2>(&mut self, module: N1, field: N2, mem: Memory)
	where
		N1: Into<Vec<u8>>,
		N2: Into<Vec<u8>>,
	{
		self.map
			.insert((module.into(), field.into()), ExternVal::Memory(mem));
	}
}

impl<T> ImportResolver for EnvironmentDefinitionBuilder<T> {
	fn resolve_func(
		&self,
		module_name: &str,
		field_name: &str,
		signature: &Signature,
	) -> Result<FuncRef, patract_wasmi::Error> {
		let key = (
			module_name.as_bytes().to_owned(),
			field_name.as_bytes().to_owned(),
		);
		let externval = self.map.get(&key).ok_or_else(|| {
			patract_wasmi::Error::Instantiation(format!(
				"Export {}:{} not found",
				module_name, field_name
			))
		})?;
		let host_func_idx = match *externval {
			ExternVal::HostFunc(ref idx) => idx,
			_ => {
				return Err(patract_wasmi::Error::Instantiation(format!(
					"Export {}:{} is not a host func",
					module_name, field_name
				)))
			}
		};
		Ok(FuncInstance::alloc_host(signature.clone(), host_func_idx.0))
	}

	fn resolve_global(
		&self,
		_module_name: &str,
		_field_name: &str,
		_global_type: &GlobalDescriptor,
	) -> Result<GlobalRef, patract_wasmi::Error> {
		Err(patract_wasmi::Error::Instantiation(format!(
			"Importing globals is not supported yet"
		)))
	}

	fn resolve_memory(
		&self,
		module_name: &str,
		field_name: &str,
		_memory_type: &MemoryDescriptor,
	) -> Result<MemoryRef, patract_wasmi::Error> {
		let key = (
			module_name.as_bytes().to_owned(),
			field_name.as_bytes().to_owned(),
		);
		let externval = self.map.get(&key).ok_or_else(|| {
			patract_wasmi::Error::Instantiation(format!(
				"Export {}:{} not found",
				module_name, field_name
			))
		})?;

		let memory = match *externval {
			ExternVal::Memory(ref m) => m,
			_ => {
				return Err(patract_wasmi::Error::Instantiation(format!(
					"Export {}:{} is not a memory",
					module_name, field_name
				)))
			}
		};
		Ok(memory.memref.clone())
	}

	fn resolve_table(
		&self,
		_module_name: &str,
		_field_name: &str,
		_table_type: &TableDescriptor,
	) -> Result<TableRef, patract_wasmi::Error> {
		Err(patract_wasmi::Error::Instantiation(format!(
			"Importing tables is not supported yet"
		)))
	}
}

pub struct Instance<T> {
	instance: ModuleRef,
	defined_host_functions: DefinedHostFunctions<T>,
}

impl<T> Instance<T> {
	pub fn new(
		code: &[u8],
		env_def_builder: &EnvironmentDefinitionBuilder<T>,
		state: &mut T,
	) -> Result<Instance<T>, Error> {
		let module = Module::from_buffer(code)
			.map_err(|_| Error::Module)?
			.try_parse_names();
		let not_started_instance =
			ModuleInstance::new(&module, env_def_builder).map_err(|_| Error::Module)?;

		let defined_host_functions = env_def_builder.defined_host_functions.clone();
		let instance = {
			let mut externals = GuestExternals {
				state,
				defined_host_functions: &defined_host_functions,
			};
			let instance = not_started_instance
				.run_start(&mut externals)
				.map_err(|_| Error::Execution)?;
			instance
		};

		Ok(Instance {
			instance,
			defined_host_functions,
		})
	}

	pub fn invoke(
		&mut self,
		name: &str,
		args: &[Value],
		state: &mut T,
	) -> Result<ReturnValue, Error> {
		let args = args
			.iter()
			.cloned()
			.map(|v| unsafe {
				let wv: wasmi::RuntimeValue = v.into();
				transmute::<wasmi::RuntimeValue, patract_wasmi::RuntimeValue>(wv)
			})
			.collect::<Vec<_>>();
		let mut externals = GuestExternals {
			state,
			defined_host_functions: &self.defined_host_functions,
		};
		let result = self.instance.invoke_export(&name, &args, &mut externals);

		match result {
			Ok(None) => Ok(ReturnValue::Unit),
			Ok(Some(val)) => unsafe {
				Ok(ReturnValue::Value(
					transmute::<patract_wasmi::RuntimeValue, wasmi::RuntimeValue>(val).into(),
				))
			},
			Err(e) => Err(match e {
				patract_wasmi::Error::Trap(t) => Error::Trap(t.into()),
				_ => Error::Execution,
			}),
		}
	}

	pub fn get_global_val(&self, name: &str) -> Option<Value> {
		let global = self.instance.export_by_name(name)?.as_global()?.get();

		Some(unsafe {
			transmute::<patract_wasmi::RuntimeValue, wasmi::RuntimeValue>(global).into()
		})
	}
}

impl Into<OutterTrap> for Trap {
	fn into(self) -> OutterTrap {
		super::Trap {
			code: match self.kind() {
				TrapKind::StackOverflow => TrapCode::StackOverflow,
				TrapKind::DivisionByZero => TrapCode::IntegerDivisionByZero,
				TrapKind::ElemUninitialized => TrapCode::BadSignature,
				TrapKind::InvalidConversionToInt => TrapCode::BadConversionToInteger,
				TrapKind::MemoryAccessOutOfBounds => TrapCode::MemoryOutOfBounds,
				TrapKind::TableAccessOutOfBounds => TrapCode::TableOutOfBounds,
				TrapKind::UnexpectedSignature => TrapCode::BadSignature,
				TrapKind::Unreachable => TrapCode::UnreachableCodeReached,
				TrapKind::Host(_) => TrapCode::HostError,
			},
			trace: self.wasm_trace().to_vec(),
		}
	}
}

impl fmt::Display for OutterTrap {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		let trace = &self.trace;
		if trace.len() == 0 {
			write!(f, "[]")?;
		} else {
			for (index, trace) in trace.iter().enumerate() {
				if index == trace.len() - 1 {
					write!(f, "\n\t╰─>")?;
				} else {
					write!(f, "\n\t|  ")?;
				}
				write!(f, "{}", trace)?;
			}
		}

		Ok(())
	}
}
