use clap::{App, Arg, SubCommand};

use crate::reset_cmd::ResetCmd;
use crate::imp_src_cmd::ImpSrcCmd;

mod reset_cmd;
mod dao;
mod error;
mod imp_src_cmd;

const APP_NAME: &str = "bm_tool";
const APP_VER_NUM: &str = "0.1.0";

#[tokio::main]
async fn main() {
    let cli_app = App::new(APP_NAME).version(APP_VER_NUM)
        .about("tools for face/car app")
        .subcommand(
            SubCommand::with_name("reset")
                .about("reset all data")
                .arg(Arg::with_name("db")
                    .short("d")
                    .long("db")
                    .takes_value(true)
                    .required(true)
                    .help("sqlite db file"))
                .arg(Arg::with_name("url_a")
                    .short("a")
                    .long("url_a")
                    .takes_value(true)
                    .required(true)
                    .help("analysis api url"))
                .arg(Arg::with_name("url_r")
                    .short("r")
                    .long("url_r")
                    .takes_value(true)
                    .required(true)
                    .help("recognition api url"))
                .arg(Arg::with_name("img_dir")
                    .short("i")
                    .long("img_dir")
                    .takes_value(true)
                    .required(true)
                    .help("image dir"))
        )
        .subcommand(
            SubCommand::with_name("imp_src")
                .about("batch create sources")
                .arg(Arg::with_name("db")
                    .short("d")
                    .long("db")
                    .default_value("../cfbm.db")
                    .required(true)
                    .help("sqlite db file"))
                .arg(Arg::with_name("url_a")
                    .short("a")
                    .long("url_a")
                    .default_value("http://localhost:7001")
                    .required(true)
                    .help("analysis api url"))
                .arg(Arg::with_name("remove")
                    .short("x")
                    .help("remove all sources"))
                .arg(Arg::with_name("config")
                    .short("c")
                    .long("config")
                    .default_value("create_src.json")
                    .required(true)
                    .help("config json"))
        );


    let cli_matches = cli_app.get_matches();

    match cli_matches.subcommand() {
        ("reset", Some(sub_matches)) => {
            let db_url = sub_matches.value_of("db").unwrap();
            let url_a = sub_matches.value_of("url_a").unwrap();
            let url_r = sub_matches.value_of("url_r").unwrap();
            let img_dir = sub_matches.value_of("img_dir").unwrap();

            let mut cmd = ResetCmd::new(db_url, url_a, url_r, img_dir);
            if let Err(e) = cmd.run_cmd().await {
                println!("error, {:?}", e);
            }
        }
        ("imp_src", Some(sub_matches)) => {
            let db_url = sub_matches.value_of("db").unwrap();
            let url_a = sub_matches.value_of("url_a").unwrap();
            let config = sub_matches.value_of("config").unwrap();
            let remove = sub_matches.is_present("remove");

            let mut cmd = ImpSrcCmd::new(db_url, url_a, config, remove);
            if let Err(e) = cmd.run_cmd().await {
                println!("error, {:?}", e);
            }
        }
        _ => {
            println!("{}", cli_matches.usage());
        }
    }
}
