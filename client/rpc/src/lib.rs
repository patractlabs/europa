// This file is part of europa
//
// Copyright 2020-2022 Patract Labs. Licensed under GPL-3.0.

mod error;

use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
};

use serde::{Deserialize, Serialize};
// Substrate
use sc_utils::mpsc::{tracing_unbounded, TracingUnboundedReceiver, TracingUnboundedSender};
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, BlockIdTo, NumberFor},
	SaturatedConversion,
};
// Local
use ec_client_api::statekv;

use crate::error::EuropaRpcError;

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NumberOrHash<B: BlockT> {
	Number(NumberFor<B>),
	Hash(B::Hash),
}

#[rpc(client, server)]
pub trait EuropaApi<B>
where
	B: BlockT,
{
	/// The rpc provide a way to produce a batch of empty block to reach target block height.
	#[method(name = "europa_forwardToHeight")]
	fn forward_to_height(&self, height: NumberFor<B>) -> RpcResult<()>;

	/// The rpc could revert current best height to the specified height which is less than current
	/// best height.
	#[method(name = "europa_backwardToHeight")]
	fn backward_to_height(&self, height: NumberFor<B>) -> RpcResult<()>;

	/// The rpc could print the modified state kvs for a specified block height or hash.
	#[method(name = "europa_modifiedStateKvs")]
	fn state_kvs(
		&self,
		number_or_hash: NumberOrHash<B>,
		child: Option<Bytes>,
	) -> RpcResult<HashMap<Bytes, Option<Bytes>>>;

	/// The rpc can get the changed state for pointed extrinsic. Notice the changed state is only
	/// for this extrinsic, may be different with the block modified state kvs, because the changed
	/// state may be modified by following extrinsics.
	#[method(name = "europa_extrinsicStateChanges")]
	fn extrinsic_changes(
		&self,
		number_or_hash: NumberOrHash<B>,
		index: u32,
	) -> RpcResult<serde_json::Value>;
}

pub enum Message<B: BlockT> {
	Forward(NumberFor<B>),
}

pub struct Europa<C, B: BlockT, Backend, S> {
	client: Arc<C>,
	backend: Arc<Backend>,
	sender: TracingUnboundedSender<Message<B>>,
	_marker: PhantomData<S>,
}

impl<C, B: BlockT, Backend, S> Clone for Europa<C, B, Backend, S> {
	fn clone(&self) -> Self {
		Self {
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
		(Self { client, backend, sender: tx, _marker: Default::default() }, rx)
	}
}

#[async_trait]
impl<C, B, Backend, S> EuropaApiServer<B> for Europa<C, B, Backend, S>
where
	C: HeaderBackend<B> + BlockIdTo<B, Error = sp_blockchain::Error> + statekv::ClientStateKv<B, S>,
	C: Send + Sync + 'static,
	B: BlockT,
	Backend: sc_client_api::backend::Backend<B> + Send + Sync + 'static,
	S: statekv::StateKv<B> + 'static,
{
	fn forward_to_height(&self, height: NumberFor<B>) -> RpcResult<()> {
		let best = self.client.info().best_number;
		if height <= best {
			return Err(EuropaRpcError::<B>::InvalidForwardHeight(height, best).into())
		}
		// height > number
		let need_more = height - best;
		let _ = self.sender.unbounded_send(Message::Forward(need_more));
		Ok(())
	}

	fn backward_to_height(&self, height: NumberFor<B>) -> RpcResult<()> {
		let best = self.client.info().best_number;
		if height >= best {
			return Err(EuropaRpcError::<B>::InvalidBackwardHeight(height, best).into())
		}
		let diff = best - height;
		self.backend.revert(diff, true).map_err(error::client_err::<B>)?;
		let state_kv = self.client.state_kv();
		let mut current = best;
		while current != height {
			state_kv.revert_all(current).map_err(|e| error::client_err::<B>(e.into()))?;
			current -= 1_u64.saturated_into();
		}

		Ok(())
	}

	fn state_kvs(
		&self,
		number_or_hash: NumberOrHash<B>,
		child: Option<Bytes>,
	) -> RpcResult<HashMap<Bytes, Option<Bytes>>> {
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
			state_kv.get_kvs_by_hash(hash).ok_or(EuropaRpcError::<B>::NoStateKvs(number_or_hash))?
		};
		let kvs = kvs.into_iter().map(|(k, v)| (Bytes(k), v.map(Bytes))).collect();

		Ok(kvs)
	}

	fn extrinsic_changes(
		&self,
		number_or_hash: NumberOrHash<B>,
		index: u32,
	) -> RpcResult<serde_json::Value> {
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
