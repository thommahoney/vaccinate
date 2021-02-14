use std::process;
use std::time::Duration;

use clap::{App, Arg};
use tokio::task;
use tokio::time::sleep;

mod config;
mod errors;
mod pushover;
mod walgreens;

#[tokio::main]
async fn main() {
    let app = App::new("Vaccinate")
        .version("0.1.0")
        .author("Thom Mahoney <mahoneyt@gmail.com>")
        .about("Checks and notifies for COVID-19 vaccine availability.")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .default_value("vaccinate.toml")
                .help("Path to vaccinate.toml configuration."),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("Enables verbose logging and error reporting."),
        )
        .get_matches();

    let config_arg = app.value_of("config");
    let debug_arg = app.is_present("debug");

    let config = match config::Config::read(config_arg, debug_arg) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Configuration error: {:?}", e);
            process::exit(1);
        }
    };

    if config.debug.unwrap() {
        println!("[debug] config = {:?}", config);
    }

    println!("Hello, {}. Let's vaccinate...", config.name);

    loop {
        let config = config.clone();

        let cvs = match config.cvs {
            Some(ref _c) => task::spawn(async {
                // let provider = cvs::Provider::new(config.clone());
                // provider.perform().await;
            }),
            None => task::spawn(async {
                println!("No CVS configuration. Moving on.");
            }),
        };

        let walgreens = match config.walgreens {
            Some(ref _w) => task::spawn(async move {
                let provider = walgreens::Provider::new(config.clone());
                provider.perform().await;
            }),
            None => task::spawn(async {
                println!("No Walgreens configuration. Moving on.");
            }),
        };

        for provider in vec![cvs, walgreens] {
            match provider.await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("unexpected error: {}", e);
                }
            }
        }

        println!("Sleeping for an hour...");
        sleep(Duration::from_secs(3600)).await;
    }
}
