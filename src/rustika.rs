use rustika::web::config::Config;
use rustika::web::response::ServerConfig;
use rustika::{Result, TikaBuilder, TikaClient};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "rustika",
    about = "A simple rust client for Tika using the standalone Tika server."
)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
enum App {
    /// See how the server is configured
    #[structopt(name = "config")]
    Config(Config),
}

fn run_config(config: &Config, client: &TikaClient) -> Result<()> {
    println!(
        "{}",
        client
            .get_json(&client.endpoint_url(config.path()))?
            .text()?
    );
    Ok(())
}

#[derive(Debug, StructOpt)]
enum Parse {
    #[structopt(name = "all")]
    All,
    #[structopt(name = "text")]
    Text,
    #[structopt(name = "meta")]
    Meta,
}

fn main() -> Result<()> {
    let app = App::from_args();

    let client = TikaBuilder::new("http://localhost:9998").build();

    match app {
        App::Config(config) => run_config(&config, &client),
        _ => Ok(()),
    }
}
