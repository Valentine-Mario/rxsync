use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, prelude::*, BufRead, Error};
use std::{fs, path::Path};

#[derive(Deserialize, Debug, Serialize)]
pub struct Config {
    pub folders: HashMap<String, String>,
    pub files: HashMap<String, String>,
}

pub enum FolderConfig {
    Add(String, String),
    Remove(String),
}

const CHECKSUM_FILE: &str = ".xsync.toml";
const IGNORE_FILE: &str = ".xsyncignore";

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

pub fn parse_checksum_config(data: String) -> Result<Config, String> {
    let raw_cfg: Result<Config, toml::de::Error> = toml::from_str(&data);
    let raw_cfg = match raw_cfg {
        Ok(raw_cfg) => raw_cfg,
        Err(_) => return Err(String::from("Error parsing config")),
    };
    Ok(raw_cfg)
}

pub fn update_folder_config(
    data: String,
    key: &str,
    path: &Path,
    action: &FolderConfig,
) -> Result<(), Error> {
    let folder_path = format!("{}/{}", path.to_str().unwrap(), CHECKSUM_FILE);
    let mut a = parse_checksum_config(data).unwrap();

    if (key != "folders") && (key != "files") {
        println!("invalid key input");
        return Ok(());
    }

    match action {
        FolderConfig::Add(key, value) => {
            if key == "folders" {
                a.folders.insert(key.to_string(), value.to_string());
                let toml_str = toml::to_string(&a).unwrap();
                fs::write(&folder_path, &toml_str)?;
                Ok(())
            } else {
                a.files.insert(key.to_string(), value.to_string());
                let toml_str = toml::to_string(&a).unwrap();
                fs::write(&folder_path, &toml_str)?;
                Ok(())
            }
        }
        FolderConfig::Remove(item) => {
            if key == "folders" {
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
