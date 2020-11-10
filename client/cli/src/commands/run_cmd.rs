// This file is part of europa which is forked form Substrate.

// Copyright (C) 2018-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// Copyright 2020-2021 patract labs. Licensed under GPL-3.0.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use regex::Regex;
use structopt::StructOpt;

use sc_cli::{arg_enums::RpcMethods, TransactionPoolParams};

use ec_service::{BasePath, TransactionPoolOptions};

use crate::config::{CliConfiguration, KeystoreParams};
use crate::params::{ImportParams, SharedParams};
use crate::{Error, Result};

/// The `run` command used to run a node.
#[derive(Debug, StructOpt)]
pub struct RunCmd {
	/// Listen to all RPC interfaces.
	///
	/// Default is local. Note: not all RPC methods are safe to be exposed publicly. Use an RPC proxy
	/// server to filter out dangerous methods. More details: https://github.com/paritytech/substrate/wiki/Public-RPC.
	/// Use `--unsafe-rpc-external` to suppress the warning if you understand the risks.
	#[structopt(long = "rpc-external")]
	pub rpc_external: bool,

	/// Listen to all RPC interfaces.
	///
	/// Same as `--rpc-external`.
	#[structopt(long)]
	pub unsafe_rpc_external: bool,

	/// RPC methods to expose.
	///
	/// - `Unsafe`: Exposes every RPC method.
	/// - `Safe`: Exposes only a safe subset of RPC methods, denying unsafe RPC methods.
	/// - `Auto`: Acts as `Safe` if RPC is served externally, e.g. when `--{rpc,ws}-external` is passed,
	///   otherwise acts as `Unsafe`.
	#[structopt(
		long,
		value_name = "METHOD SET",
		possible_values = &RpcMethods::variants(),
		case_insensitive = true,
		default_value = "Auto",
		verbatim_doc_comment
	)]
	pub rpc_methods: RpcMethods,

	/// Listen to all Websocket interfaces.
	///
	/// Default is local. Note: not all RPC methods are safe to be exposed publicly. Use an RPC proxy
	/// server to filter out dangerous methods. More details: https://github.com/paritytech/substrate/wiki/Public-RPC.
	/// Use `--unsafe-ws-external` to suppress the warning if you understand the risks.
	#[structopt(long = "ws-external")]
	pub ws_external: bool,

	/// Listen to all Websocket interfaces.
	///
	/// Same as `--ws-external` but doesn't warn you about it.
	#[structopt(long = "unsafe-ws-external")]
	pub unsafe_ws_external: bool,

	/// Specify IPC RPC server path
	#[structopt(long = "ipc-path", value_name = "PATH")]
	pub ipc_path: Option<String>,

	/// Specify HTTP RPC server TCP port.
	#[structopt(long = "rpc-port", value_name = "PORT")]
	pub rpc_port: Option<u16>,

	/// Specify WebSockets RPC server TCP port.
	#[structopt(long = "ws-port", value_name = "PORT")]
	pub ws_port: Option<u16>,

	/// Maximum number of WS RPC server connections.
	#[structopt(long = "ws-max-connections", value_name = "COUNT")]
	pub ws_max_connections: Option<usize>,

	/// Specify browser Origins allowed to access the HTTP & WS RPC servers.
	///
	/// A comma-separated list of origins (protocol://domain or special `null`
	/// value). Value of `all` will disable origin validation. Default is to
	/// allow localhost and https://polkadot.js.org origins. When running in
	/// --dev mode the default is to allow all origins.
	#[structopt(long = "rpc-cors", value_name = "ORIGINS", parse(try_from_str = parse_cors))]
	pub rpc_cors: Option<Cors>,

	/// The human-readable name for this node.
	///
	/// The node name will be reported to the telemetry server, if enabled.
	#[structopt(long = "name", value_name = "NAME")]
	pub name: Option<String>,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub shared_params: SharedParams,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub import_params: ImportParams,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub pool_config: TransactionPoolParams,

	/// Enable authoring even when offline.
	#[structopt(long = "force-authoring")]
	pub force_authoring: bool,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub keystore_params: KeystoreParams,

	/// The size of the instances cache for each runtime.
	///
	/// The default value is 8 and the values higher than 256 are ignored.
	#[structopt(long)]
	pub max_runtime_instances: Option<usize>,

	/// Run a temporary node.
	///
	/// A temporary directory will be created to store the configuration and will be deleted
	/// at the end of the process.
	///
	/// Note: the directory is random per process execution. This directory is used as base path
	/// which includes: database, node key and keystore.
	#[structopt(long, conflicts_with_all = &["base-path", "workspace"])]
	pub tmp: bool,
}
impl CliConfiguration for RunCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}

	fn import_params(&self) -> Option<&ImportParams> {
		Some(&self.import_params)
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		Some(&self.keystore_params)
	}

	fn node_name(&self) -> Result<String> {
		let name: String = match self.name.as_ref() {
			Some(name) => name.to_string(),
			None => "europa-sandbox".to_string(),
		};

		is_node_name_valid(&name).map_err(|msg| {
			Error::Input(format!(
				"Invalid node name '{}'. Reason: {}. If unsure, use none.",
				name, msg
			))
		})?;

		Ok(name)
	}

	fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
		Ok(self.ws_max_connections)
	}

	fn rpc_cors(&self) -> Result<Option<Vec<String>>> {
		Ok(self.rpc_cors.clone().unwrap_or_else(|| Cors::All).into())
	}

	fn rpc_http(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		let interface = rpc_interface(
			self.rpc_external,
			self.unsafe_rpc_external,
			self.rpc_methods,
			true, // todo check this
		)?;

		Ok(Some(SocketAddr::new(
			interface,
			self.rpc_port.unwrap_or(default_listen_port),
		)))
	}

	fn rpc_ipc(&self) -> Result<Option<String>> {
		Ok(self.ipc_path.clone())
	}

	fn rpc_ws(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		let interface = rpc_interface(
			self.ws_external,
			self.unsafe_ws_external,
			self.rpc_methods,
			true, // todo check this
		)?;

		Ok(Some(SocketAddr::new(
			interface,
			self.ws_port.unwrap_or(default_listen_port),
		)))
	}

	fn rpc_methods(&self) -> Result<ec_service::RpcMethods> {
		Ok(self.rpc_methods.into())
	}

	fn transaction_pool(&self) -> Result<TransactionPoolOptions> {
		Ok(self.pool_config.transaction_pool())
	}

	fn base_path(&self) -> Result<Option<BasePath>> {
		Ok(if self.tmp {
			Some(BasePath::new_temp_dir()?)
		} else {
			self.shared_params().base_path()
		})
	}
}

/// The maximum number of characters for a node name.
pub(crate) const NODE_NAME_MAX_LENGTH: usize = 64;

/// Check whether a node name is considered as valid.
pub fn is_node_name_valid(_name: &str) -> std::result::Result<(), &str> {
	let name = _name.to_string();
	if name.chars().count() >= NODE_NAME_MAX_LENGTH {
		return Err("Node name too long");
	}

	let invalid_chars = r"[\\.@]";
	let re = Regex::new(invalid_chars).unwrap();
	if re.is_match(&name) {
		return Err("Node name should not contain invalid chars such as '.' and '@'");
	}

	let invalid_patterns = r"(https?:\\/+)?(www)+";
	let re = Regex::new(invalid_patterns).unwrap();
	if re.is_match(&name) {
		return Err("Node name should not contain urls");
	}

	Ok(())
}

fn rpc_interface(
	is_external: bool,
	is_unsafe_external: bool,
	rpc_methods: RpcMethods,
	is_validator: bool,
) -> Result<IpAddr> {
	if is_external && is_validator && rpc_methods != RpcMethods::Unsafe {
		return Err(Error::Input(
			"--rpc-external and --ws-external options shouldn't be \
		used if the node is running as a validator. Use `--unsafe-rpc-external` \
		or `--rpc-methods=unsafe` if you understand the risks. See the options \
		description for more information."
				.to_owned(),
		));
	}

	if is_external || is_unsafe_external {
		if rpc_methods == RpcMethods::Unsafe {
			log::warn!(
				"It isn't safe to expose RPC publicly without a proxy server that filters \
			available set of RPC methods."
			);
		}

		Ok(Ipv4Addr::UNSPECIFIED.into())
	} else {
		Ok(Ipv4Addr::LOCALHOST.into())
	}
}

/// CORS setting
///
/// The type is introduced to overcome `Option<Option<T>>`
/// handling of `structopt`.
#[derive(Clone, Debug)]
pub enum Cors {
	/// All hosts allowed.
	All,
	/// Only hosts on the list are allowed.
	List(Vec<String>),
}

impl From<Cors> for Option<Vec<String>> {
	fn from(cors: Cors) -> Self {
		match cors {
			Cors::All => None,
			Cors::List(list) => Some(list),
		}
	}
}

/// Parse cors origins.
fn parse_cors(s: &str) -> std::result::Result<Cors, Box<dyn std::error::Error>> {
	let mut is_all = false;
	let mut origins = Vec::new();
	for part in s.split(',') {
		match part {
			"all" | "*" => {
				is_all = true;
				break;
			}
			other => origins.push(other.to_owned()),
		}
	}

	Ok(if is_all {
		Cors::All
	} else {
		Cors::List(origins)
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn tests_node_name_good() {
		assert!(is_node_name_valid("short name").is_ok());
	}

	#[test]
	fn tests_node_name_bad() {
		assert!(is_node_name_valid(
			"very very long names are really not very cool for the ui at all, really they're not"
		)
		.is_err());
		assert!(is_node_name_valid("Dots.not.Ok").is_err());
		assert!(is_node_name_valid("http://visit.me").is_err());
		assert!(is_node_name_valid("https://visit.me").is_err());
		assert!(is_node_name_valid("www.visit.me").is_err());
		assert!(is_node_name_valid("email@domain").is_err());
	}
}
