use std::env;
use std::process;
use std::time::Duration;

use tokio::task;
use tokio::time::sleep;

mod config;
mod walgreens;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        eprintln!("No arguments are accepted at this time.");
        process::exit(1);
    }

    // TODO: accept --config flag
    let config = config::Config::read(None).unwrap();

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
                Ok(_) => { },
                Err(e) => {
                    eprintln!("unexpected error: {}", e);
                }
            }
        }

        println!("Sleeping for an hour...");
        sleep(Duration::from_secs(3600)).await;
    }
}
