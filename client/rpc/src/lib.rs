mod error;

use serde::{Deserialize, Serialize};
use std::{collections::hash_map::HashMap, sync::Arc};

use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_runtime::traits::{Block as BlockT, BlockIdTo, Header};
use sp_utils::mpsc::{tracing_unbounded, TracingUnboundedReceiver, TracingUnboundedSender};

use ec_client_api::statekv;

use error::EuropaRpcError;
use sp_runtime::generic::BlockId;

pub enum Message<B: BlockT> {
	Forward(NumberOf<B>),
}

pub struct Europa<C, B: BlockT, Backend, S> {
	client: Arc<C>,
	backend: Arc<Backend>,
	sender: TracingUnboundedSender<Message<B>>,
	_marker: std::marker::PhantomData<S>,
}

impl<C, B: BlockT, Backend, S> Clone for Europa<C, B, Backend, S> {
	fn clone(&self) -> Self {
		Europa {
			client: self.client.clone(),
			backend: self.backend.clone(),
			sender: self.sender.clone(),
			_marker: self._marker.clone(),
		}
	}
}

impl<C, B: BlockT, Backend, S> Europa<C, B, Backend, S> {
	/// Create new `Contracts` with the given reference to the client.
	pub fn new(
		client: Arc<C>,
		backend: Arc<Backend>,
	) -> (Self, TracingUnboundedReceiver<Message<B>>) {
		let (tx, rx) = tracing_unbounded("mpsc_europa_rpc");
		(
			Europa {
				client,
				backend,
				sender: tx,
				_marker: Default::default(),
			},
			rx,
		)
	}
}

#[rpc]
pub trait EuropaApi<B>
where
	B: BlockT,
{
	/// The rpc provide a way to produce a batch of empty block to reach target block height.
	#[rpc(name = "europa_forwardToHeight")]
	fn forward_to_height(&self, height: NumberOf<B>) -> Result<()>;

	/// The rpc could revert current best height to the specified height which is less than current best height.
	#[rpc(name = "europa_backwardToHeight")]
	fn backward_to_height(&self, height: NumberOf<B>) -> Result<()>;

	/// The rpc could print the modified state kvs for a specified block height or hash.
	#[rpc(name = "europa_modifiedStateKvs")]
	fn state_kvs(
		&self,
		number_or_hash: NumberOrHash<B>,
		child: Option<Bytes>,
	) -> Result<HashMap<Bytes, Option<Bytes>>>;

	/// The rpc can get the changed state for pointed extrinsic. Notice the changed state is only for this extrinsic, may be different with the block modified state kvs, because the changed state may be modified by following extrinsics.
	#[rpc(name = "europa_extrinsicStateChanges")]
	fn extrinsic_changes(
		&self,
		number_or_hash: NumberOrHash<B>,
		index: u32,
	) -> Result<serde_json::Value>;
}

type NumberOf<B> = <<B as BlockT>::Header as Header>::Number;

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum NumberOrHash<B: BlockT> {
	Number(NumberOf<B>),
	Hash(B::Hash),
}

impl<C, B, Backend, S> EuropaApi<B> for Europa<C, B, Backend, S>
where
	C: HeaderBackend<B> + BlockIdTo<B, Error = sp_blockchain::Error> + statekv::ClientStateKv<B, S>,
	C: Send + Sync + 'static,
	B: BlockT,
	Backend: sc_client_api::backend::Backend<B> + Send + Sync + 'static,
	S: statekv::StateKv<B> + 'static,
{
	fn forward_to_height(&self, height: NumberOf<B>) -> Result<()> {
		let best = self.client.info().best_number;
		if height <= best {
			return Err(EuropaRpcError::<B>::InvalidForwardHeight(height, best).into());
		}
		// height > number
		let need_more = height - best;
		let _ = self.sender.unbounded_send(Message::Forward(need_more));
		Ok(())
	}

	fn backward_to_height(&self, height: NumberOf<B>) -> Result<()> {
		let best = self.client.info().best_number;
		if height >= best {
			return Err(EuropaRpcError::<B>::InvalidBackwardHeight(height, best).into());
		}
		let diff = best - height;
		self.backend
			.revert(diff, true)
			.map_err(error::client_err::<B>)?;
		Ok(())
	}
	fn state_kvs(
		&self,
		number_or_hash: NumberOrHash<B>,
		child: Option<Bytes>,
	) -> Result<HashMap<Bytes, Option<Bytes>>> {
		let id = number_or_hash.clone();
		let hash = match id {
			NumberOrHash::Hash(hash) => hash,
			NumberOrHash::Number(num) => self
				.client
				.to_hash(&BlockId::Number(num))
				.map_err(error::client_err::<B>)?
				.ok_or(EuropaRpcError::<B>::InvalidBlockId(number_or_hash.clone()))?,
		};

		let state_kv = self.client.state_kv();
		let kvs = if let Some(child) = child {
			// todo treat child as a prefix in future or split this rpc to two interface
			state_kv
				.get_child_kvs_by_hash(hash, &child)
				.ok_or(EuropaRpcError::<B>::NoChildStateKvs(number_or_hash, child))?
		} else {
			state_kv
				.get_kvs_by_hash(hash)
				.ok_or(EuropaRpcError::<B>::NoStateKvs(number_or_hash))?
		};
		let kvs = kvs
			.into_iter()
			.map(|(k, v)| (Bytes(k), v.map(Bytes)))
			.collect();

		Ok(kvs)
	}
	fn extrinsic_changes(
		&self,
		number_or_hash: NumberOrHash<B>,
		index: u32,
	) -> Result<serde_json::Value> {
		use ec_basic_authorship::Event;
		let id = number_or_hash.clone();
		let number = match id {
			NumberOrHash::Hash(hash) => self
				.client
				.to_number(&BlockId::Hash(hash))
				.map_err(error::client_err::<B>)?
				.ok_or(EuropaRpcError::<B>::InvalidBlockId(number_or_hash.clone()))?,
			NumberOrHash::Number(num) => num,
		};
		let json = self
			.client
			.state_kv()
			.get_extrinsic_changes(number, index)
			.ok_or(EuropaRpcError::<B>::NoExtrinsic(number_or_hash, index))?;
		let r: Vec<Event> = serde_json::from_str(&json).expect("should not fail.");
		Ok(serde_json::json!(r))
	}
}
