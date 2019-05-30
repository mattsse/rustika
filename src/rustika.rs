use rustika::client::{TikaServerFile, Verbosity};
use rustika::web::config::Config;
use rustika::web::response::ServerConfig;
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

fn server_demo() -> Result<()> {
    let mut client = TikaBuilder::default()
        .server_verbosity(Verbosity::Verbose)
        .start_server()?;

    //    let addr = net::SocketAddr::new(net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1)), 9998);
    //    let handle = jar.start_server(&addr, Verbosity::Silent)?;

    //    client.server_handle = Some(handle);

    //    let target = client.download_server_jar().expect("Failed to download");
    //
    //    match app {
    //        App::Config(config) => run_config(&config, &client),
    //        _ => Ok(()),
    //    }

    println!("sleeping...");
    std::thread::sleep(std::time::Duration::from_secs(45));

    Ok(())
}

fn main() -> Result<()> {
    use which;
    pretty_env_logger::init();
    //    let app = App::from_args();

    //        println!("{:?}", which::which("tika-rest-server").unwrap());

    server_demo();

    Ok(())
}
