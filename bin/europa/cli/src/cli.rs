use sc_cli::RunCmd;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Cli {
	#[structopt(flatten)]
	pub run: RunCmd,
}
