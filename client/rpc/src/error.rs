use jsonrpc_core as rpc;

use sp_runtime::traits::Block as BlockT;

use crate::NumberOf;

#[derive(Debug)]
pub enum EuropaRpcError<B: BlockT> {
	InvalidForwardHeight(NumberOf<B>, NumberOf<B>),
}

impl<B: BlockT> From<EuropaRpcError<B>> for rpc::Error {
	fn from(e: EuropaRpcError<B>) -> Self {
		match e {
			EuropaRpcError::InvalidForwardHeight(forward, best) => rpc::Error {
				code: rpc::ErrorCode::InvalidParams,
				message: format!(
					"forward height should more than current best: forward: {}|best: {}",
					forward, best
				)
				.into(),
				data: None,
			},
		}
	}
}
