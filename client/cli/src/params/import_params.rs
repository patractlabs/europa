// This file is part of europa which is forked form Substrate.

// Copyright (C) 2018-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// Copyright 2020-2022 Patract Labs. Licensed under GPL-3.0.

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

use sc_cli::DatabaseParams;

/// Parameters for block import.
#[derive(Debug, StructOpt)]
pub struct ImportParams {
	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub database_params: DatabaseParams,

	/// Specify the state cache size.
	#[structopt(
		long = "state-cache-size",
		value_name = "Bytes",
		default_value = "67108864"
	)]
	pub state_cache_size: usize,
}

impl ImportParams {
	/// Specify the state cache size.
	pub fn state_cache_size(&self) -> usize {
		self.state_cache_size
	}
}
