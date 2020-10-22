use structopt::StructOpt;
use sc_cli::RunCmd;

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(flatten)]
    pub run: RunCmd,
}
