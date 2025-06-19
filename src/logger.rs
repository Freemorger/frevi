use std::{fs::File, io::Write};

use chrono::{DateTime, Local};

#[derive(Debug)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    CriticalError,
    PluginFault,
}

#[derive(Debug)]
pub struct Logger {
    log_file: File,
}

impl Logger {
    pub fn new(path: String) -> Result<Logger, String> {
        let log: File = match File::create(path) {
            Ok(f) => f,
            Err(e) => {
                return Err(e.to_string());
            }
        };
        let res = Logger { log_file: log };
        Ok(res)
    }
    pub fn log_msg(&mut self, ltype: LogLevel, msg: String) -> Result<(), String> {
        let now: DateTime<Local> = Local::now();
        let res_msg: String = match ltype {
            LogLevel::Info => {
                format!(
                    "{}\tInfo: {}\n",
                    now.clone().format("%Y-%m-%d %H:%M:%S"),
                    msg
                )
            }
            LogLevel::Warning => {
                format!(
                    "{}\tWarning: {}\n",
                    now.clone().format("%Y-%m-%d %H:%M:%S"),
                    msg
                )
            }
            LogLevel::Error => {
                format!(
                    "{}\tERROR: {}\n",
                    now.clone().format("%Y-%m-%d %H:%M:%S"),
                    msg
                )
            }
            LogLevel::CriticalError => {
                format!(
                    "{}\tCRITICAL: {}\n",
                    now.clone().format("%Y-%m-%d %H:%M:%S"),
                    msg
                )
            }
            LogLevel::PluginFault => {
                format!(
                    "{}\tPLUGIN ERROR: {}\n",
                    now.clone().format("%Y-%m-%d %H:%M:%S"),
                    msg
                )
            }
        };
        match self.log_file.write_all(&res_msg.into_bytes()) {
            Ok(_) => {}
            Err(e) => {
                return Err(e.to_string());
            }
        }
        Ok(())
    }
}
