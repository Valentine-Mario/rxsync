use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, prelude::*, BufRead, Error};
use std::{fs, path::Path, path::PathBuf};

#[derive(Deserialize, Debug, Serialize)]
pub struct Config {
    pub folders: HashMap<String, String>,
    pub files: HashMap<String, String>,
}

#[derive(Debug)]
pub enum FolderConfig {
    Add(String, String),
    Remove(String),
}

pub const CHECKSUM_FILE: &str = ".xsync.toml";
pub const IGNORE_FILE: &str = ".xsyncignore";

pub fn create_checksum_file(path: &Path) -> Result<(), Error> {
    let folder_path = format!("{}/{}", path.to_str().unwrap(), CHECKSUM_FILE);
    if !Path::new(&folder_path).exists() {
        let mut file = fs::File::create(folder_path)?;
        let config = Config {
            folders: HashMap::new(),
            files: HashMap::new(),
        };
        let toml = toml::to_string(&config).unwrap();
        file.write_all(toml.as_bytes())?;
    }
    Ok(())
}

pub fn get_ignore_file(path: &Path) -> Result<Vec<String>, Error> {
    let folder_path = format!("{}/{}", path.to_str().unwrap(), IGNORE_FILE);
    let mut all_lines = vec![];
    if Path::new(&folder_path).exists() {
        if let Ok(lines) = read_lines(folder_path) {
            for line in lines {
                let str_path = path.to_str().unwrap();
                if str_path.starts_with("./") {
                    let result = str_path.replace("./", "");
                    let resolved_path = Path::new("").join(result).join(line?);
                    all_lines.push(String::from(resolved_path.to_str().unwrap()))
                } else {
                    let resolved_path = Path::new("").join(path).join(line?);
                    all_lines.push(String::from(resolved_path.to_str().unwrap()))
                }
            }
            return Ok(all_lines);
        }
    }
    Ok(vec![])
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
where
    P: AsRef<Path>,
{
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn read_checksum_file(path: &Path) -> Result<String, Error> {
    let folder_path = format!("{}/{}", path.to_str().unwrap(), CHECKSUM_FILE);
    let data = fs::read_to_string(folder_path)?;
    Ok(data)
}

pub fn parse_checksum_config(data: &String) -> Result<Config, String> {
    let raw_cfg: Result<Config, toml::de::Error> = toml::from_str(&data);
    let raw_cfg = match raw_cfg {
        Ok(raw_cfg) => raw_cfg,
        Err(_) => return Err(String::from("Error parsing config")),
    };
    Ok(raw_cfg)
}

pub fn update_folder_config(
    key_config: &str,
    path: &Path,
    action: &FolderConfig,
) -> Result<(), Error> {
    let folder_path = format!("{}/{}", path.to_str().unwrap(), CHECKSUM_FILE);
    let cfg_data = read_checksum_file(path)?;
    let mut a = parse_checksum_config(&cfg_data).unwrap();

    if (key_config != "folders") && (key_config != "files") {
        println!("invalid key input");
        return Ok(());
    }
    match action {
        FolderConfig::Add(key, value) => {
            if key_config == "folders" {
                a.folders.insert(key.to_string(), value.to_string());
                let toml_str = toml::to_string(&a).unwrap();
                fs::write(&folder_path, &toml_str)?;
            } else {
                a.files.insert(key.to_string(), value.to_string());
                let toml_str = toml::to_string(&a).unwrap();
                fs::write(&folder_path, &toml_str)?;
            }
            Ok(())
        }
        FolderConfig::Remove(item) => {
            if key_config == "folders" {
                a.folders.remove(item);
                let toml_str = toml::to_string(&a).unwrap();
                fs::write(&folder_path, &toml_str)?;
            } else {
                a.files.remove(item);
                let toml_str = toml::to_string(&a).unwrap();
                fs::write(&folder_path, &toml_str)?;
            }

            Ok(())
        }
    }
}

pub fn get_items_to_delete(
    config_state: &HashMap<String, String>,
    item_list: &Vec<PathBuf>,
) -> Vec<String> {
    let mut return_vec: Vec<String> = vec![];
    //if an item exist on the config state
    //but no longer on the item list
    //mark as delete

    for (key, _) in config_state.into_iter() {
        if !item_list.contains(&PathBuf::from(key)) {
            return_vec.push(key.to_string())
        }
    }
    return_vec
}

pub fn get_items_to_upload(
    config_state: &HashMap<String, String>,
    item_list: &Vec<PathBuf>,
) -> Vec<String> {
    //if an item exist in memory but not config state
    //mark to upload
    let mut return_vec: Vec<String> = vec![];
    for item in item_list {
        if !config_state.contains_key(item.to_str().unwrap()) {
            return_vec.push(item.to_str().unwrap().to_string())
        }
    }
    return_vec
}
