#![allow(dead_code)]
use std::fs::{self, File,OpenOptions};
use std::io::{self, Read, Write};
use toml::{self};
use serde_derive::Deserialize;
use regex::Regex;
use lazy_static::lazy_static;
use clap::{self, command, arg};
use std::process::{self};
// #read epublish's config toml
// #set default value according to config

// get now work dir
// if a workspace xxx 
// get the input args
// fork function
// - increasing version: 有时候增加一个major有时候增加一个minor有时候只加一个patch
//   get  current version
//   add  version
//   save version
// - increasing and publish

// for example: cargo epublish --config custom
// fn read_custom_toml(filename: &str) -> fs::File {
//     match read_only_file(filename) {
//         Ok(f) => f,
//         Err(e) => panic!("{e}")
//     }
// }
static mut CHANGED_FLAG: bool = false;
lazy_static! {
    static ref RE: Regex = Regex::new(r"^\[(.+)-(.+)\]\s*$").unwrap();
    // static ref RE_V: Regex = Regex::new("^version = \"\\d\\.\\d\\.\\d\"$").unwrap();
    static ref RE_V: Regex = Regex::new(r"^\s*version\s*=\s*\u0022\d\.\d\.\d+\s*\u0022$").unwrap();
}
fn change_flag(flag: bool) {
    unsafe {
        if flag != CHANGED_FLAG {
            CHANGED_FLAG = flag
        }
    }
}
fn read_only_file(filename: &str) -> Result<File, io::Error> {  
    OpenOptions::new().read(true).open(filename)
}
fn resolve_config_name(filename: &str) -> String {
    if filename.ends_with(".toml") {
        String::from(filename)
    } else {
        String::from(filename) + ".toml"
    }
}
fn read_toml(filename: &str) -> String {
    let filename = resolve_config_name(filename);
    let mut file = match read_only_file(&filename) {
        Ok(f) => f,
        Err(e) => panic!("{e}")
    };
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    drop(file);
    resolve_connector_in_config(buf)
}
fn resolve_connector_in_config(file_content: String) -> String {
    file_content.lines().map(|line| {
        if line.is_empty() {
            String::new()
        } else if RE.is_match(line) {
            change_flag(true);
            let caps = RE.captures(line).unwrap();
            format!("[{}_{}]",caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str())
        } else {
            line.to_string()
        }
    }).collect::<Vec<String>>().join("\r\n")
}

fn easy_replace_version(filename: &str, new: &str) {
    let origin = String::from(filename) + ".origin";
    fs::rename(filename, &origin).unwrap();
    let mut file = read_only_file(&origin).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    drop(file);
    let content = content.lines().map(|line| {
        if line.is_empty() {
            String::new()
        } else if RE_V.is_match(line) {
            change_flag(false);
            format!("version = \"{}\"", new)
        } else {
            line.to_string()
        }
    }).collect::<Vec<String>>().join("\r\n");
    let mut file = File::create(filename).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    match file.flush() {
        Ok(_) => fs::remove_file(origin).unwrap(),
        Err(e) => panic!("{e}")
    }
}
#[derive(Deserialize, Debug)]
struct Package {
    version: String
}
#[derive(Deserialize, Debug)]
struct MaxVersions {
    // major: usize,
    minor: usize,
    patch: usize
}
#[derive(Deserialize, Debug)]
struct Config {
    package: Package,
    max_versions: MaxVersions
}
#[repr(usize)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum Grade {
    Patch,
    Minor,
    Major
}

impl ToString for Grade {
    fn to_string(&self) -> String {
        match *self as usize {
            0 => String::from("patch"),
            1 => String::from("minor"),
            2 => String::from("marjor"),
            _ => panic!("unknown grade")
        } 
    }
}
impl TryFrom<&str> for Grade {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, String> {
        match s {
            "patch" => Ok(Grade::Patch),
            "minor" => Ok(Grade::Minor),
            "major" => Ok(Grade::Major),
            _ => Err(String::from("unknown grade"))
        }
    }
}

fn upgrade_patch_minor(version: &mut Vec<usize>, max_version: usize, index: usize) -> Option<()> {
    let p = version[index] + 1;
    let mut flag = None;
    if index == 1 {version[index + 1] = 0};
    version[index] = if p == max_version {
        flag = Some(());
        0
    } else {
        p
    };
    flag
} 

fn upgrade_major(version: &mut Vec<usize>) {
    version[0] = version[0] + 1;
    version[1] = {version[2] = 0; version[2]};
}
fn resolve_upgrade(version: &mut Vec<usize>, method: &str, max_version: &MaxVersions) {
    let method = Grade::try_from(method).unwrap();
    if method == Grade::Patch {
        if let Some(_) = upgrade_patch_minor(version, max_version.patch,2) {
            if let Some(_) = upgrade_patch_minor(version, max_version.minor,1)  {
                upgrade_major(version)
            }
        }
    } else if method == Grade::Minor {
        if let Some(_) = upgrade_patch_minor(version,max_version.minor,1)  {
            upgrade_major(version)
        }
    } else {
        upgrade_major(version)
    }
}

fn publish() {
    match process::Command::new("git")
    .args(["add", "."])
    .output() {
        Ok(_) => {
            process::Command::new("cargo")
            .args(["publish"])
            .output()
            .expect("epublish error at the moment of 'cargo publish'");
        },
        Err(_) => {
            eprintln!("'git add .' error");
            process::exit(256)
        }
    }
    
}
//cli功能实现
fn main() {
    let matches = command!()
        .arg(arg!(
            [upgrade_name] "Optional name to operate on, you can choose [publish/patch/minor/major]"
        ).possible_values(["patch", "minor", "major", "publish"])
        )
        .arg(
            arg!(
               -u --upgrade <VALUE> "set the upgrade method [patch/minor/major]"
            )
            // We don't have syntax yet for optional options, so manually calling `required`
            .required(false)
        )
        .arg(
            arg!(
               -c --config <VALUE> "Sets a custom config file"
            ).
            default_value("Cargo.toml")
            .required(false)
        )
        .get_matches();
    //get the toml config content
    let config_name = matches.value_of("config").unwrap();
    let toml_content = read_toml(config_name);
    //get the version field
    let config: Config = toml::from_str(&toml_content).unwrap();
    let version = &config.package.version;
    let mut version = version.split(".").into_iter().map(|v| v.parse().unwrap()).collect::<Vec<usize>>();
    let mut need_publish = false;
    if let Some(upgrade_name) = matches.value_of("upgrade_name") {
        match upgrade_name {
            "patch" | "minor" | "major" => resolve_upgrade(&mut version, upgrade_name, &config.max_versions),
            "publish" => {
                if let Some(upgrade) = matches.value_of("upgrade") {
                    need_publish = true;
                    resolve_upgrade(&mut version, upgrade, &config.max_versions)
                } else {
                    eprintln!("[error] must definate a --upgrade");
                    process::exit(256);
                }
            },
            _ => { process::exit(256); }
        }
    } else {
        if let Some(upgrade) = matches.value_of("upgrade") {
            resolve_upgrade(&mut version, upgrade, &config.max_versions)
        } else {
            eprintln!("[error] must choose a upgrade [patch/minor/major]");
            process::exit(256);
        }
    }
    let version = version.into_iter().map(|v| v.to_string()).collect::<Vec<String>>();
    let version = version.join(".");
    easy_replace_version(config_name, &version);
    if need_publish {
        publish();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_version_patch() {
        let max_version = MaxVersions{
            minor: 10,
            patch: 10
        };
        let version = "0.0.9";
        let mut version = version.split(".").into_iter().map(|v| v.parse().unwrap()).collect::<Vec<usize>>();
        resolve_upgrade(&mut version, "patch", &max_version);
        let version = version.into_iter().map(|v| v.to_string()).collect::<Vec<String>>();
        let version = version.join(".");
        assert_eq!("0.1.0", version)
    }
    #[test]
    fn test_version_minor() {
        let max_version = MaxVersions{
            minor: 10,
            patch: 10
        };
        let version = "0.9.9";
        let mut version = version.split(".").into_iter().map(|v| v.parse().unwrap()).collect::<Vec<usize>>();
        resolve_upgrade(&mut version, "minor", &max_version);
        let version = version.into_iter().map(|v| v.to_string()).collect::<Vec<String>>();
        let version = version.join(".");
        assert_eq!("1.0.0", version)
    }#[test]
    fn test_version_major() {
        let max_version = MaxVersions{
            minor: 10,
            patch: 10
        };
        let version = "0.9.9";
        let mut version = version.split(".").into_iter().map(|v| v.parse().unwrap()).collect::<Vec<usize>>();
        resolve_upgrade(&mut version, "major", &max_version);
        let version = version.into_iter().map(|v| v.to_string()).collect::<Vec<String>>();
        let version = version.join(".");
        assert_eq!("1.0.0", version)
    }
}