use std::convert::{TryFrom, TryInto};
use std::sync::Arc;

use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, Header as HeaderT},
};

use pallet_contracts_primitives::Code;
pub use pallet_contracts_rpc_runtime_api::ContractsApi as ContractsRuntimeApi;

use pallet_contracts::NestedRuntime;
use pallet_contracts_rpc::Weight;

use ec_client_api::statekv::StateKv;
pub use europa_runtime::runtime_api::ContractsExtApi as ContractsExtRuntimeApi;
use europa_runtime::{AccountId, Balance, Runtime};

const RUNTIME_ERROR: i64 = 1;

/// A rough estimate of how much gas a decent hardware consumes per second,
/// using native execution.
/// This value is used to set the upper bound for maximal contract calls to
/// prevent blocking the RPC for too long.
///
/// As 1 gas is equal to 1 weight we base this on the conducted benchmarks which
/// determined runtime weights:
/// https://github.com/paritytech/substrate/pull/5446
const GAS_PER_SECOND: Weight = 1_000_000_000_000;

/// The maximum amount of weight that the call and instantiate rpcs are allowed to consume.
/// This puts a ceiling on the weight limit that is supplied to the rpc as an argument.
const GAS_LIMIT: Weight = 5 * GAS_PER_SECOND;

/// A struct that encodes RPC parameters required for a call to a smart-contract.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct CallRequest<AccountId> {
	origin: AccountId,
	dest: AccountId,
	value: NumberOrHex,
	gas_limit: NumberOrHex,
	input_data: Bytes,
}

/// A struct that encodes RPC parameters required to instantiate a new smart-contract.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct InstantiateRequest<AccountId, Hash> {
	origin: AccountId,
	endowment: NumberOrHex,
	gas_limit: NumberOrHex,
	code: Code<Hash>,
	data: Bytes,
	salt: Bytes,
}

/// ContractsExt RPC methods.
#[rpc]
pub trait ContractsExtApi<BlockHash, BlockNumber> {
	/// Executes a call to a contract.
	///
	/// This call is performed locally without submitting any transactions. Thus executing this
	/// won't change any state. Nonetheless, the calling state-changing contracts is still possible.
	///
	/// This method is useful for calling getter-like methods on contracts.
	#[rpc(name = "contractsExt_call")]
	fn call(
		&self,
		call_request: CallRequest<AccountId>,
		at: Option<BlockHash>,
	) -> Result<serde_json::Value>;

	/// Instantiate a new contract.
	///
	/// This call is performed locally without submitting any transactions. Thus the contract
	/// is not actually created.
	///
	/// This method is useful for UIs to dry-run contract instantiations.
	#[rpc(name = "contractsExt_instantiate")]
	fn instantiate(
		&self,
		instantiate_request: InstantiateRequest<AccountId, BlockHash>,
		at: Option<BlockHash>,
	) -> Result<serde_json::Value>;

	#[rpc(name = "contractsExt_tracing")]
	fn tracing(&self, number: BlockNumber, index: u32) -> Result<serde_json::Value>;
}

/// An implementation of contract specific RPC methods.
pub struct ContractsExt<C, B, S> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<(B, S)>,
}
impl<C, B, S> ContractsExt<C, B, S> {
	/// Create new `Contracts` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		ContractsExt {
			client,
			_marker: Default::default(),
		}
	}
}

impl<C, B, S> ContractsExtApi<<B as BlockT>::Hash, <<B as BlockT>::Header as HeaderT>::Number>
	for ContractsExt<C, B, S>
where
	B: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<B> + HeaderBackend<B>,
	C: ec_client_api::statekv::ClientStateKv<B, S>,
	C::Api: ContractsExtRuntimeApi<
		B,
		AccountId,
		Balance,
		<<B as BlockT>::Header as HeaderT>::Number,
		<B as BlockT>::Hash,
	>,
	S: ec_client_api::statekv::StateKv<B> + 'static,
{
	fn call(
		&self,
		call_request: CallRequest<AccountId>,
		at: Option<<B as BlockT>::Hash>,
	) -> Result<serde_json::Value> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

		let CallRequest {
			origin,
			dest,
			value,
			gas_limit,
			input_data,
		} = call_request;

		let value: Balance = decode_hex(value, "balance")?;
		let gas_limit: Weight = decode_hex(gas_limit, "weight")?;
		limit_gas(gas_limit)?;

		let (exec_result, trace) = api
			.call(&at, origin, dest, value, gas_limit, input_data.to_vec())
			.map_err(runtime_error_into_rpc_err)?;

		let mut t: NestedRuntime<Runtime> =
			serde_json::from_str(&trace).expect("trace string must be a valid json");
		trim_gas_trace(&mut t);
		Ok(json!({
			"result": exec_result,
			"trace": t,
		}))
	}

	fn instantiate(
		&self,
		instantiate_request: InstantiateRequest<AccountId, <B as BlockT>::Hash>,
		at: Option<<B as BlockT>::Hash>,
	) -> Result<serde_json::Value> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

		let InstantiateRequest {
			origin,
			endowment,
			gas_limit,
			code,
			data,
			salt,
		} = instantiate_request;

		let endowment: Balance = decode_hex(endowment, "balance")?;
		let gas_limit: Weight = decode_hex(gas_limit, "weight")?;
		limit_gas(gas_limit)?;

		let (exec_result, trace) = api
			.instantiate(
				&at,
				origin,
				endowment,
				gas_limit,
				code,
				data.to_vec(),
				salt.to_vec(),
			)
			.map_err(runtime_error_into_rpc_err)?;

		let mut t: NestedRuntime<Runtime> =
			serde_json::from_str(&trace).expect("trace string must be a valid json");
		trim_gas_trace(&mut t);
		Ok(json!({
			"result": exec_result,
			"trace": t,
		}))
	}

	fn tracing(
		&self,
		number: <<B as BlockT>::Header as HeaderT>::Number,
		index: u32,
	) -> Result<serde_json::Value> {
		let state_kv = self.client.state_kv();
		let trace = state_kv
			.get_contract_tracing(number, index)
			.ok_or(ContractExtError::<B>::NoTracing(number, index))?;
		let mut t: NestedRuntime<Runtime> =
			serde_json::from_str(&trace).expect("trace string must be a valid json");
		trim_gas_trace(&mut t);
		Ok(json!({
			"trace": t,
		}))
	}
}

fn trim_gas_trace(trace: &mut NestedRuntime<Runtime>) {
	let env_trace = trace.modify_env_trace();
	env_trace.0.retain(|item| {
		if let pallet_contracts::env_trace::EnvTrace::Gas(_) = item {
			false
		} else {
			true
		}
	});
	for sub_item in trace.nests_mut().iter_mut() {
		trim_gas_trace(sub_item);
	}
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> Error {
	Error {
		code: ErrorCode::ServerError(RUNTIME_ERROR),
		message: "Runtime error".into(),
		data: Some(format!("{:?}", err).into()),
	}
}

fn decode_hex<H: std::fmt::Debug + Copy, T: TryFrom<H>>(from: H, name: &str) -> Result<T> {
	from.try_into().map_err(|_| Error {
		code: ErrorCode::InvalidParams,
		message: format!("{:?} does not fit into the {} type", from, name),
		data: None,
	})
}

fn limit_gas(gas_limit: Weight) -> Result<()> {
	if gas_limit > GAS_LIMIT {
		Err(Error {
			code: ErrorCode::InvalidParams,
			message: format!(
				"Requested gas limit is greater than maximum allowed: {} > {}",
				gas_limit, GAS_LIMIT
			),
			data: None,
		})
	} else {
		Ok(())
	}
}

#[derive(Debug)]
pub enum ContractExtError<B: BlockT> {
	NoTracing(<<B as BlockT>::Header as HeaderT>::Number, u32),
}

impl<B: BlockT> From<ContractExtError<B>> for jsonrpc_core::Error {
	fn from(e: ContractExtError<B>) -> Self {
		match e {
			ContractExtError::<B>::NoTracing(number, index) => jsonrpc_core::Error {
				code: jsonrpc_core::ErrorCode::InvalidParams,
				message: format!(
					"No contract tracing for this extrinsic index: number:{:}|index:{:}",
					number, index,
				)
				.into(),
				data: None,
			},
		}
	}
}
