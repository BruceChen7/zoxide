use crate::app::{Add, Run};
use crate::config;
use crate::db::DatabaseFile;
use crate::util;

use anyhow::{bail, Result};

use std::path::Path;

// 实现结构体方法
impl Run for Add {
    fn run(&self) -> Result<()> {
        // These characters can't be printed cleanly to a single line, so they
        // can cause confusion when writing to fzf / stdout.
        const EXCLUDE_CHARS: &[char] = &['\n', '\r'];

        let data_dir = config::data_dir()?;
        let exclude_dirs = config::exclude_dirs()?;
        let max_age = config::maxage()?; // ？遇到错误立刻返回
        let now = util::current_time()?;

        let mut db = DatabaseFile::new(data_dir);
        let mut db = db.open()?;

        // 迭代
        for path in self.paths.iter() {
            // 解决符号链接
            let path = if config::resolve_symlinks() {
                util::canonicalize(path)
            } else {
                util::resolve_path(path)
            }?;
            // 转成相关的path
            let path = util::path_to_str(&path)?;

            // Ignore path if it contains unsupported characters, or if it's in
            // the exclude list.
            // 忽略相关的路径
            // 忽略相关的目录
            if path.contains(EXCLUDE_CHARS) || exclude_dirs.iter().any(|glob| glob.matches(path)) {
                continue;
            }
            // 如果不是目录
            if !Path::new(path).is_dir() {
                bail!("not a directory: {}", path);
            }
            db.add(path, now);
        }

        if db.modified {
            db.age(max_age);
            db.save()?;
        }

        Ok(())
    }
}
