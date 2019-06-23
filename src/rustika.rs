use rustika::client::Verbosity::Verbose;
use rustika::client::{TikaServerFile, Verbosity};
use rustika::web::config::Config;
use rustika::web::response::ServerConfig;
use rustika::web::translate::{Language, Translator};
use rustika::{Result, TikaBuilder, TikaClient};
use std::net;
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
    /// translate documents
    #[structopt(name = "translate")]
    Translate,
}

fn run_config(config: &Config, client: &TikaClient) -> Result<()> {
    println!("{}", client.get_json(config.path())?.text()?);
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
    use std::fs;
    use which;
    pretty_env_logger::init();
    //    let app = App::from_args();

    //    let client = TikaBuilder::client_only("http://localhost:9998")?.build();

    let client = TikaBuilder::default()
        .translator(Translator::Google)
        .start_server()?;
    //        println!("{:?}", which::which("tika-rest-server").unwrap());
    let content = fs::read("Cargo.toml")?;
    //    println!("{:?}", content.len());
    //    let mime = client.detect_language(content);

    let val = client.translate("bonjour mon ami!", "fr", "en");

    println!("{:?}", val);

    //    std::thread::sleep(std::time::Duration::from_secs(45));

    Ok(())
}
