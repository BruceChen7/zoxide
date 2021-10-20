use crate::db::Rank;

use anyhow::{bail, Context, Result};
use dirs_next as dirs;
use glob::Pattern;

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

// 获取path 路径的目录
pub fn data_dir() -> Result<PathBuf> {
    // 获取环境变量
    let path = match env::var_os("_ZO_DATA_DIR") {
        Some(path) => PathBuf::from(path),
        None => match dirs::data_local_dir() {
            Some(mut path) => {
                path.push("zoxide");
                path
            }
            None => bail!("could not find data directory, please set _ZO_DATA_DIR manually"),
        },
    };

    // 返回path
    Ok(path)
}

pub fn echo() -> bool {
    // 获取当前进程的环境变量
    match env::var_os("_ZO_ECHO") {
        Some(var) => var == "1",
        None => false,
    }
}

pub fn exclude_dirs() -> Result<Vec<Pattern>> {
    match env::var_os("_ZO_EXCLUDE_DIRS") {
        Some(paths) => env::split_paths(&paths)
            .map(|path| {
                let pattern = path.to_str().context("invalid unicode in _ZO_EXCLUDE_DIRS")?;
                // 支持pattern模式
                Pattern::new(pattern)
                    .with_context(|| format!("invalid glob in _ZO_EXCLUDE_DIRS: {}", pattern))
            })
            .collect(),
        None => {
            let pattern = (|| {
                // 获取home 目录
                let home = dirs::home_dir()?;
                // home目录
                let home = home.to_str()?;
                let home = Pattern::escape(home);
                Pattern::new(&home).ok()
            })();
            Ok(pattern.into_iter().collect())
        }
    }
}

// 根据平台
pub fn fzf_opts() -> Option<OsString> {
    // 获取
    env::var_os("_ZO_FZF_OPTS")
}

pub fn maxage() -> Result<Rank> {
    match env::var_os("_ZO_MAXAGE") {
        Some(maxage) => {
            let maxage = maxage.to_str().context("invalid unicode in _ZO_MAXAGE")?;
            let maxage = maxage
                .parse::<u64>()
                .with_context(|| format!("unable to parse _ZO_MAXAGE as integer: {}", maxage))?;
            Ok(maxage as Rank)
        }
        None => Ok(10000.0),
    }
}

pub fn resolve_symlinks() -> bool {
    // 是否返回符号变量
    match env::var_os("_ZO_RESOLVE_SYMLINKS") {
        Some(var) => var == "1",
        None => false,
    }
}
