use structopt::StructOpt;

use ec_cli::{RunCmd, StateKvCmd, WorkspaceCmd};

#[derive(Debug, StructOpt)]
pub struct Cli {
	#[structopt(flatten)]
	pub run: RunCmd,

	#[structopt(subcommand)]
	pub subcommand: Option<Subcommand>,
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
	/// Print modified stored state kvs for a block.
	StateKv(StateKvCmd),

	/// Related to workspace operation.
	Workspace(WorkspaceCmd),
}
