use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "rustika",
    about = "A simple rust client for Tika using the standalone Tika server."
)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
pub(crate) struct App {
    /// Activate debug mode
    #[structopt(short = "d", long = "debug")]
    debug: bool,
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() {
    let _app = App::from_args();
}
