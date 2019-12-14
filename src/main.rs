#[macro_use]
extern crate clap;

mod app_core;

use clap::{App, Arg};
use env_logger::Builder;
use log::LevelFilter;
use log::{debug, error, info, trace};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("config json file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("filename")
                .short("n")
                .long("filename")
                .help("output files name")
                .default_value("TODO")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("type")
                .short("t")
                .long("type")
                .help("output files type")
                .possible_value("raw")
                .possible_value("markdown")
                .possible_value("json")
                .default_value("raw")
                .takes_value(true),
        )
        .subcommand(
            App::new("onedir")
                .about("collects comments in one specified project directory")
                .arg(Arg::with_name("path").short("p").help("path to directory").index(1)),
        )
        .subcommand(
            App::new("workspace")
                .about("collects comments from all directories in one worspace")
                .arg(Arg::with_name("path").short("p").help("path to directory").index(1)),
        )
        .get_matches();

    // verbosity
    let level_filter = match matches.occurrences_of("verbose") {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    Builder::new().filter_level(level_filter).init();
    debug!("Debug logs enabled");

    // app settings
    let mut settings = config::Config::default();
    let home_dir = match std::env::var("HOME") {
        Ok(p) => format!("{}/workspace", p),
        Err(_) => "/root".to_owned(),
    };
    settings.set_default("workspace", vec![home_dir]).unwrap();
    settings.set_default("prefix", vec!["TODO"]).unwrap();
    settings.set_default("comment_symbol", vec!["//"]).unwrap();
    let outfile_name = matches.value_of("filename").unwrap_or("TODO");
    let outfile_type = matches.value_of("type").unwrap_or("raw");
    let mut outfile: HashMap<String, config::Value> = HashMap::new();
    outfile.insert("name".to_string(), config::Value::new(None, outfile_name));
    outfile.insert("type".to_string(), config::Value::new(None, outfile_type));
    settings.set_default("outfile", outfile)?;

    // TDOD: separate config per workspace

    if let Some(c) = matches.value_of("config") {
        match settings.merge(config::File::with_name(c)) {
            Ok(_s) => info!("Config file: {}", c),
            Err(e) => error!("Config file {} not found: {:?}", c, e),
        }
    }
    trace!("App config:\n{:#?}", settings);

    match matches.subcommand_name() {
        Some("onedir") => {
            // TODO searching comments in one project directory
            unimplemented!();
        }
        Some("workspace") => {
            // TODO searching comments in one workspace directory
            unimplemented!();
        }
        Some(_) => unreachable!(),
        None => info!("No subcommand used"),
    };

    app_core::run_app(&settings)
}
