#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate dirs;
extern crate env_logger;
extern crate hdfs;
extern crate serde;
extern crate serde_json;
extern crate toml;
extern crate walk;

mod config;
mod err;
mod walk_hdfs;

use clap::{App, Arg, SubCommand};
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::BufRead;
use std::io::BufReader;
use std::path::{Path, PathBuf};

fn ls(configPath: PathBuf, gateway: Option<&str>, path: PathBuf) {
    let hdfs_fs = hdfs::hdfs::get_hdfs(configPath, gateway, None).unwrap();

    let walk = walk::walk::WalkBuilder::new(walk_hdfs::HdfsFileSystem::new(hdfs_fs))
        .with_path(path)
        .build()
        .unwrap();
    println!("{:?}", walk);
    for file in walk {
        let path = file.unwrap();
        let path_str = path.to_str().unwrap();
        println!("{}", path_str);
    }
}

const DEFAULT_PATH_STR: &str = ".hdfsrc";

fn home_config() -> Option<config::Config> {
    let home = dirs::home_dir();

    if home.is_none() {
        return None;
    }
    let mut home = home.unwrap();
    home.push(DEFAULT_PATH_STR);

    if !home.exists() {
        return None;
    }

    let mut f =
        fs::File::open(home.as_os_str()).expect(&format!("Error while opening {}", home.display()));

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect(&format!("Error while reading {}", home.display()));

    let config: config::Config = toml::from_str(&contents).expect("Format error in config file");

    Some(config)
}

fn main() {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::Builder::from_env(env).init();

    let yaml = load_yaml!("cli.yml");
    let home_config = home_config();

    let matches = App::from_yaml(yaml).get_matches();
    let hadoop_install_path = env::var("HADOOP_INSTALL").map(|e| Some(e)).unwrap_or(None);
    let hadoop_default_gateway = env::var("GATEWAY_DEFAULT").map(|e| Some(e)).unwrap_or(None);
    println!("{:?}", home_config);
    let config = matches
        .value_of("config")
        .or(home_config
            .as_ref()
            .and_then(|c| c.hadoop.as_ref())
            .and_then(|h| h.config_path.as_ref().map(String::as_ref)))
        .or(hadoop_install_path.as_ref().map(String::as_ref))
        .expect(&format!(
            "No hadoop config path has been found. Please set hadoop config path in ~/{}",
            DEFAULT_PATH_STR
        ));

    let config = PathBuf::from(config);
    let gateway = matches
        .value_of("gateway")
        .or(hadoop_default_gateway.as_ref().map(String::as_ref))
        .or(home_config
            .as_ref()
            .and_then(|c| c.gateway.as_ref())
            .and_then(|g| g.default.as_ref().map(String::as_ref)));

    if let Some(matches) = matches.subcommand_matches("ls") {
        let path = matches.value_of("PATH").unwrap();
        let path = PathBuf::from(path);
        ls(config, gateway, path)
    }
}
