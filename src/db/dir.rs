use anyhow::{bail, Context, Result};
use bincode::Options as _;
use serde::{Deserialize, Serialize};

use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Deserialize, Serialize)]
pub struct DirList<'a>(#[serde(borrow)] pub Vec<Dir<'a>>);

impl DirList<'_> {
    const VERSION: u32 = 3;

    pub fn new() -> DirList<'static> {
        // DirList 列表
        DirList(Vec::new())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<DirList> {
        // Assume a maximum size for the database. This prevents bincode from
        // throwing strange errors when it encounters invalid data.
        // 32 MiB
        const MAX_SIZE: u64 = 32 << 20;
        // 反序列化
        let deserializer = &mut bincode::options().with_fixint_encoding().with_limit(MAX_SIZE);

        // Split bytes into sections.
        let version_size = deserializer.serialized_size(&Self::VERSION).unwrap() as _;
        // 内容长度不够
        if bytes.len() < version_size {
            // 提前返回Error
            bail!("could not deserialize database: corrupted data");
        }
        // 前面是version，后面是
        let (bytes_version, bytes_dirs) = bytes.split_at(version_size);

        // Deserialize sections.
        (|| {
            let version = deserializer.deserialize(bytes_version)?;
            match version {
                // 返回目录信息
                // 静态变量
                Self::VERSION => Ok(deserializer.deserialize(bytes_dirs)?),
                // 其他
                version => {
                    bail!("unsupported version (got {}, supports {})", version, Self::VERSION,)
                }
            }
        })()
        .context("could not deserialize database")
    }

    // 返回多个字节，序列化
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        (|| -> bincode::Result<_> {
            // Preallocate buffer with combined size of sections.
            let version_size = bincode::serialized_size(&Self::VERSION)?;
            let dirs_size = bincode::serialized_size(&self)?;
            let buffer_size = version_size + dirs_size;
            // 创建buffer
            let mut buffer = Vec::with_capacity(buffer_size as _);

            // Serialize sections into buffer.
            // 序列化到buffer
            bincode::serialize_into(&mut buffer, &Self::VERSION)?;
            bincode::serialize_into(&mut buffer, &self)?;
            Ok(buffer)
        })()
        .context("could not serialize database")
    }
}

// 实现解引用 trait
impl<'a> Deref for DirList<'a> {
    type Target = Vec<Dir<'a>>;

    // 静态类型
    fn deref(&self) -> &Self::Target {
        // 返回可读引用
        &self.0
    }
}

// 实现DerefMut
impl<'a> DerefMut for DirList<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> From<Vec<Dir<'a>>> for DirList<'a> {
    fn from(dirs: Vec<Dir<'a>>) -> Self {
        DirList(dirs)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dir<'a> {
    #[serde(borrow)]
    // path
    // Copy On write的智能指针，适用于读多写少
    pub path: Cow<'a, str>,
    pub rank: Rank,
    pub last_accessed: Epoch,
}

impl Dir<'_> {
    // 获取每个目录的得分
    pub fn score(&self, now: Epoch) -> Rank {
        const HOUR: Epoch = 60 * 60;
        const DAY: Epoch = 24 * HOUR;
        const WEEK: Epoch = 7 * DAY;

        // The older the entry, the lesser its importance.
        let duration = now.saturating_sub(self.last_accessed);
        // 在一个小时内访问，排名靠前
        if duration < HOUR {
            self.rank * 4.0
        } else if duration < DAY {
            self.rank * 2.0
        } else if duration < WEEK {
            self.rank * 0.5
        } else {
            self.rank * 0.25
        }
    }

    pub fn display(&self) -> DirDisplay {
        DirDisplay { dir: self }
    }

    pub fn display_score(&self, now: Epoch) -> DirDisplayScore {
        DirDisplayScore { dir: self, now }
    }
}

pub struct DirDisplay<'a> {
    dir: &'a Dir<'a>,
}

// 返回Display trait
impl Display for DirDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // 写下当前的路径
        write!(f, "{}", self.dir.path)
    }
}

pub struct DirDisplayScore<'a> {
    dir: &'a Dir<'a>,
    now: Epoch,
}

impl Display for DirDisplayScore<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let score = self.dir.score(self.now);
        let score = if score > 9999.0 {
            9999
        } else if score > 0.0 {
            score as u32
        } else {
            0
        };
        write!(f, "{:>4} {}", score, self.dir.path)
    }
}

pub type Rank = f64;
pub type Epoch = u64;

#[cfg(test)]
mod tests {
    use super::{Dir, DirList};

    use std::borrow::Cow;

    #[test]
    fn zero_copy() {
        let dirs = DirList(vec![Dir { path: "/".into(), rank: 0.0, last_accessed: 0 }]);

        let bytes = dirs.to_bytes().unwrap();
        let dirs = DirList::from_bytes(&bytes).unwrap();

        for dir in dirs.iter() {
            assert!(matches!(dir.path, Cow::Borrowed(_)))
        }
    }
}
