mod error;

use std::sync::Arc;

use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

use sp_blockchain::HeaderBackend;
use sp_runtime::traits::{Block as BlockT, Header};
use sp_utils::mpsc::{tracing_unbounded, TracingUnboundedReceiver, TracingUnboundedSender};

use error::EuropaRpcError;

pub enum Message<B: BlockT> {
	Forward(NumberOf<B>),
}

pub struct Europa<C, B: BlockT, Backend> {
	client: Arc<C>,
	backend: Arc<Backend>,
	sender: TracingUnboundedSender<Message<B>>,
}

impl<C, B: BlockT, Backend> Clone for Europa<C, B, Backend> {
	fn clone(&self) -> Self {
		Europa {
			client: self.client.clone(),
			backend: self.backend.clone(),
			sender: self.sender.clone(),
		}
	}
}

impl<C, B: BlockT, Backend> Europa<C, B, Backend> {
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
			},
			rx,
		)
	}
}

#[rpc]
pub trait EuropaApi<BlockNumber> {
	#[rpc(name = "europa_forwardToHeight")]
	fn forward_to_height(&self, height: BlockNumber) -> Result<()>;

	#[rpc(name = "europa_backwardToHeight")]
	fn backward_to_height(&self, height: BlockNumber) -> Result<()>;
}

type NumberOf<B> = <<B as BlockT>::Header as Header>::Number;

impl<C, B, Backend> EuropaApi<NumberOf<B>> for Europa<C, B, Backend>
where
	C: HeaderBackend<B>,
	C: Send + Sync + 'static,
	B: BlockT,
	Backend: sc_client_api::backend::Backend<B> + Send + Sync + 'static,
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
		self.backend.revert(diff, true);
		Ok(())
	}
}
