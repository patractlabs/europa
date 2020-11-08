mod error;

use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
	#[rpc(name = "europa_forwardToHeight")]
	fn forward_to_height(&self, height: NumberOf<B>) -> Result<()>;

	#[rpc(name = "europa_backwardToHeight")]
	fn backward_to_height(&self, height: NumberOf<B>) -> Result<()>;

	#[rpc(name = "europa_modifiedStateKvs")]
	fn state_kvs(
		&self,
		number_or_hash: NumberOrHash<B>,
		child: Option<Bytes>,
	) -> Result<Vec<(Bytes, Option<Bytes>)>>;
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
	) -> Result<Vec<(Bytes, Option<Bytes>)>> {
		let hash = match number_or_hash {
			NumberOrHash::Hash(hash) => hash,
			NumberOrHash::Number(num) => self
				.client
				.to_hash(&BlockId::Number(num))
				.map_err(error::client_err::<B>)?
				.ok_or(EuropaRpcError::<B>::InvalidBlockNumber(num))?,
		};

		let state_kv = self.client.state_kv();
		Ok(vec![])
	}
}
