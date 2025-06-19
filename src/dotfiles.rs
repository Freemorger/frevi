use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use dirs;

#[derive(Debug)]
pub struct FreviConfig {
    pub cfg_path: PathBuf,
    pub autoplugs: Vec<String>,
    defsh: (Option<String>, Option<String>), // first for windows, second for unix-like
}

impl FreviConfig {
    pub fn new() -> FreviConfig {
        let plugsv: Vec<String> = Vec::new();
        let defshells: (Option<String>, Option<String>) = (None, None);
        let cfgpath = PathBuf::new();
        FreviConfig {
            autoplugs: plugsv,
            defsh: defshells,
            cfg_path: cfgpath,
        }
    }
    pub fn read_cfg(&mut self) -> Result<(), String> {
        let home_dir = match dirs::home_dir() {
            Some(path) => path,
            None => {
                return Err("Can't get home dir!".to_string());
            }
        };
        let cfg_dir = home_dir.join(".frevi/");
        self.cfg_path = cfg_dir.clone();
        match cfg_dir.try_exists() {
            Ok(true) => {}
            Ok(false) => {
                if let Err(e) = std::fs::create_dir(cfg_dir.clone()) {
                    return Err(e.to_string());
                }
            }
            Err(e) => {
                return Err(e.to_string());
            }
        }
        let autoload_file_path = cfg_dir.clone().join("autoplug");
        match autoload_file_path.try_exists() {
            Ok(true) => {
                let autoload_file = match File::open(autoload_file_path) {
                    Ok(f) => f,
                    Err(e) => {
                        return Err(e.to_string());
                    }
                };
                let reader = BufReader::new(autoload_file);
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            if l.starts_with(';') {
                                continue;
                            }
                            self.autoplugs.push(l);
                        }
                        Err(e) => {
                            self.autoplugs.clear();
                            return Err(e.to_string());
                        }
                    }
                }
            }
            Ok(false) => match File::create(autoload_file_path) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e.to_string());
                }
            },
            Err(e) => {
                return Err(e.to_string());
            }
        }

        Ok(())
    }
}
