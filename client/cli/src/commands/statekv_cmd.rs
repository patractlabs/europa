use std::fmt::Debug;

use bytes::Bytes;
use structopt::StructOpt;

use sp_runtime::traits::Block as BlockT;

use sc_cli::{BlockNumberOrHash, CliConfiguration, ImportParams, SharedParams};

use ec_client_api::statekv;

#[derive(Debug, StructOpt)]
pub struct StateKvCmd {
	/// Block hash or number
	#[structopt(value_name = "HASH or NUMBER")]
	pub input: BlockNumberOrHash,

	#[structopt(long = "child", value_name = "CHILD HASH", parse(try_from_str = parse_bytes))]
	pub child: Option<Bytes>,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub shared_params: SharedParams,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub import_params: ImportParams,
}

impl StateKvCmd {
	/// Run the check-block command
	pub fn run<B, C, S>(&self, client: C) -> sc_cli::Result<()>
	where
		B: BlockT,
		S: statekv::StateKv<B>,
		C: statekv::ClientStateKv<B, S>,
	{
		let state_kv = client.state_kv();
		Ok(())
	}
}
impl CliConfiguration for StateKvCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}

	fn import_params(&self) -> Option<&ImportParams> {
		Some(&self.import_params)
	}
}

fn parse_bytes(s: &str) -> std::result::Result<Bytes, String> {
	if !s.starts_with("0x") {
		return Err(format!(
			"child bytes should be hex string and start with '0x'"
		));
	}
	let b = hex::decode(s).map_err(|e| e.to_string())?;
	Ok(b.into())
}
