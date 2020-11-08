// This file is part of europa which is forked form Substrate.
// Copyright (C) 2020 Parity Technologies (UK) Ltd.
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

use structopt::StructOpt;

use sc_cli::{Error, Result};

use ec_service::config::PruningMode;

/// Parameters to define the pruning mode
#[derive(Debug, StructOpt)]
pub struct PruningParams {
	/// Specify the state pruning mode, a number of blocks to keep or 'archive'.
	///
	/// Default is to keep all block states if the node is running as a
	/// validator (i.e. 'archive').
	#[structopt(long = "pruning", value_name = "PRUNING_MODE")]
	pub pruning: Option<String>,
}

impl PruningParams {
	/// Get the pruning value from the parameters
	pub fn pruning(&self, _unsafe_pruning: bool) -> Result<PruningMode> {
		Ok(match &self.pruning {
			Some(ref s) if s == "archive" => PruningMode::ArchiveAll,
			None => PruningMode::ArchiveAll,
			Some(s) => PruningMode::keep_blocks(
				s.parse()
					.map_err(|_| Error::Input("Invalid pruning mode specified".to_string()))?,
			),
		})
	}
}

impl From<sc_cli::PruningParams> for PruningParams {
	fn from(p: sc_cli::PruningParams) -> Self {
		PruningParams { pruning: p.pruning }
	}
}
