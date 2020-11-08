use std::fmt::Debug;
use std::str::FromStr;

use bytes::Bytes;
use structopt::StructOpt;

use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, Header as HeaderT},
};

use sc_cli::{BlockNumberOrHash, CliConfiguration, Error as CliError, ImportParams, SharedParams, Role};

use ec_client_api::statekv;

use log::info;

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
		B::Hash: FromStr,
		<B::Hash as FromStr>::Err: Debug,
		<<B::Header as HeaderT>::Number as FromStr>::Err: Debug,
		S: statekv::StateKv<B>,
		C: statekv::ClientStateKv<B, S>,
	{
		let state_kv = client.state_kv();
		let id = self.input.parse::<B>().map_err(CliError::Input)?;
		let hash = match id {
			BlockId::Hash(hash) => hash,
			BlockId::Number(num) => state_kv.get_hash(num).ok_or(CliError::Input(format!(
				"do not have block hash for this block number: {}",
				num
			)))?,
		};

		let kvs = state_kv
			.get_kvs_by_hash(hash)
			.ok_or(CliError::Input(format!(
				"do not have state for this block hash: {:?}",
				hash
			)))?;
		info!("modified state for block:{:?}", hash);
		for (k, v) in kvs {
			info!(
				"	key:{:}|value:{:}",
				hex::encode(k),
				v.map(hex::encode).unwrap_or("[DELETED]".to_string())
			);
		}
		if let Some(child) = self.child.as_ref() {
			let kvs = state_kv
				.get_child_kvs_by_hash(hash, &child)
				.ok_or(CliError::Input(format!(
					"do not have state for this child:{:} in block hash:{:?}",
					hex::encode(child),
					hash,
				)))?;

			info!(
				"modified child state for block:{:?}|child:{:}",
				hash,
				hex::encode(&child)
			);
			for (k, v) in kvs {
				info!(
					"	key:{:}|value:{:}",
					hex::encode(k),
					v.map(hex::encode).unwrap_or("[DELETED]".to_string())
				);
			}
		}
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

	fn role(&self, _is_dev: bool) -> sc_cli::Result<Role> {
		Ok(Role::Authority { sentry_nodes: vec![] })
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
