mod app;

use clap::*;
use env_logger::Builder;
use log::{debug, error, info, LevelFilter};

// TODO: Add github travis CI for testing and build

fn main() -> anyhow::Result<()> {
    // TODO: use strcut witch Clap derive
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
                .default_value("$HOME/.config/todocol/settings.json")
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
            Arg::with_name("format")
                .short("f")
                .long("format")
                .help("output files format")
                .possible_value("txt")
                .possible_value("markdown")
                .default_value("txt")
                .takes_value(true),
        )
        .get_matches();

    // TODO: generate zsh completion

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
    settings.set_default("prefix", vec!["TODO"]).unwrap();
    settings.set_default("comment_symbol", vec!["//"]).unwrap();
    settings.set_default("outfile.name", "TODO").unwrap();
    settings.set_default("outfile.format", "txt").unwrap();

    // // TODO: default config in project dir ex: .todocol.json

    let config_file = matches.value_of("config").unwrap_or("$HOME/.config/todocol/settings.json");
    let config_file = shellexpand::env(config_file)?;
    info!("Config file: {}", config_file);

    if let Err(err) = settings.merge(config::File::with_name(&config_file)) {
        error!("Loading settings from file failed: {}", err)
    }

    if matches.occurrences_of("format") == 1 {
        if let Some(format) = matches.value_of("format") {
            settings.set("outfile.format", format).unwrap();
        }
    }

    if matches.occurrences_of("filename") == 1 {
        if let Some(name) = matches.value_of("filename") {
            settings.set("outfile.name", name).unwrap();
        }
    }

    // TODO: use positional argument to set project dir

    let pwd = shellexpand::env("$PWD")?;
    app::run(settings, &pwd)
}
