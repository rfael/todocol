use config::Config;
use log::{debug, error, warn};
use std::path::PathBuf;
use std::str::FromStr;

use todocol::TodoCollector;

pub fn swap_env(input: &str) -> String {
    if input.contains('$') {
        match shellexpand::env(input) {
            Ok(c) => c.to_string(),
            Err(err) => {
                error!("Can not expand env variables in {} directory: {}", &input, err);
                String::new()
            }
        }
    } else {
        input.to_owned()
    }
}

fn todo_collector_set_config(todo_collector: &mut TodoCollector, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let prefixes = config.get::<Vec<String>>("prefix")?;
    let comment_symbols = config.get::<Vec<String>>("comment_symbol")?;
    let outfile_name = config.get::<String>("outfile.name")?;
    let outfile_format = config.get::<String>("outfile.format")?;
    let source_extensions = config
        .get::<Vec<String>>("source_extension")
        .unwrap_or_else(|_| vec!["rs".into()]);
    let ignore = config.get::<Vec<String>>("ignore").unwrap_or_else(|_| Vec::new());

    debug!("prefixes: {:#?}", prefixes);
    debug!("comment_symbols: {:#?}", comment_symbols);
    debug!("source_extensions: {:#?}", source_extensions);
    debug!("outfile: {} in {} format", outfile_name, outfile_format);
    debug!("ignore list: {:#?}", ignore);

    todo_collector.set_prefixes(prefixes);
    todo_collector.set_source_extensions(source_extensions);
    todo_collector.set_comment_symbols(comment_symbols);
    todo_collector.set_outfile_name(&outfile_name);
    todo_collector.set_outfile_format(&outfile_format);
    todo_collector.set_ignore(ignore);

    todo_collector.add_ignore(".vscode");
    todo_collector.add_ignore(".git");
    todo_collector.add_ignore(".gitignore");
    todo_collector.add_ignore(".gitmodules");

    Ok(())
}

fn project_dirs(workspace_dir: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let workspace_path = match PathBuf::from_str(workspace_dir) {
        Ok(w) => w,
        Err(_err) => {
            error!("'{}' is invalid workspace path", workspace_dir);
            return result;
        }
    };

    let entry_iter = match workspace_path.read_dir() {
        Ok(ei) => ei,
        Err(err) => {
            warn!("Can not read dir {}: {}", workspace_dir, err);
            return result;
        }
    };

    for entry in entry_iter {
        if let Ok(e) = entry {
            let path = e.path();
            if path.is_dir() {
                result.push(path)
            }
        }
    }

    result
}

pub fn run_app_project(config: &Config, project_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = swap_env(project_dir);
    let mut todo_collector = TodoCollector::new();
    todo_collector_set_config(&mut todo_collector, config)?;

    todo_collector.set_project_dir(&project_dir)?;
    todo_collector.collect_project();
    todo_collector.save_comments()?;

    Ok(())
}

pub fn run_app_workspace(config: &Config, workspace_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let workspace_dir = swap_env(workspace_dir);
    let mut todo_collector = TodoCollector::new();
    todo_collector_set_config(&mut todo_collector, config)?;

    for pd in project_dirs(&workspace_dir) {
        if let Err(err) = todo_collector.set_project_dir(&pd) {
            error!("Set project dir error: {}", err);
            continue;
        }

        todo_collector.collect_project();

        if let Err(err) = todo_collector.save_comments() {
            error!("Saving comments error: {}", err);
            continue;
        }
    }

    Ok(())
}

pub fn run_app(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut workspaces = config.get::<Vec<String>>("workspace")?;
    workspaces = workspaces.iter().map(|w| swap_env(w)).filter(|w| !w.is_empty()).collect();
    debug!("workspaces: {:#?}", workspaces);

    let mut todo_collector = TodoCollector::new();
    todo_collector_set_config(&mut todo_collector, config)?;

    let mut project_paths = Vec::new();
    for w in &workspaces {
        project_paths.extend(project_dirs(w))
    }

    for pd in project_paths {
        if let Err(err) = todo_collector.set_project_dir(&pd) {
            error!("Set project dir error: {}", err);
            continue;
        }

        todo_collector.collect_project();

        if let Err(err) = todo_collector.save_comments() {
            error!("Saving comments error: {}", err);
            continue;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use shellexpand;

    #[test]
    fn env_swap() {
        let p = "$HOME/pictures";
        assert_eq!("/home/rafau/pictures", shellexpand::env(p).unwrap());
    }
}
