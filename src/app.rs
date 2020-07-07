use config::Config;
use log::debug;

use todocol::TodoCollector;

fn todo_collector_set_config(todo_collector: &mut TodoCollector, config: Config) -> anyhow::Result<()> {
    let prefixes = config.get::<Vec<String>>("prefix")?;
    let outfile_name = config.get::<String>("outfile.name")?;
    let outfile_format = config.get::<String>("outfile.format")?;
    let ignore = config.get::<Vec<String>>("ignore").unwrap_or_else(|_| Vec::new());

    debug!("prefixes: {:#?}", prefixes);
    debug!("outfile: {} in {} format", outfile_name, outfile_format);
    debug!("ignore list: {:#?}", ignore);

    todo_collector.set_prefixes(prefixes);
    todo_collector.set_result_file_name(&outfile_name);
    todo_collector.set_result_file_format(&outfile_format);
    todo_collector.set_ignore(ignore);

    // TODO: add wildcards support to ignore files
    todo_collector.add_ignore(".vscode");
    todo_collector.add_ignore(".git");
    todo_collector.add_ignore(".gitignore");
    todo_collector.add_ignore(".gitmodules");

    Ok(())
}

pub fn run(config: Config, project_dir: &str) -> anyhow::Result<()> {
    let mut todo_collector = TodoCollector::new();

    todo_collector_set_config(&mut todo_collector, config)?;

    todo_collector.collect_from_project(&project_dir)?;

    Ok(())
}

#[cfg(test)]
mod tests {}
