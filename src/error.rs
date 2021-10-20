use anyhow::{bail, Context, Result};

use std::fmt::{self, Display, Formatter};
use std::io;

// Custom error type for early exit.
#[derive(Debug)]
pub struct SilentExit {
    pub code: i32,
}

impl Display for SilentExit {
    fn fmt(&self, _: &mut Formatter) -> fmt::Result {
        // 啥都不返回
        Ok(())
    }
}

pub trait BrokenPipeHandler {
    fn pipe_exit(self, device: &str) -> Result<()>;
}

impl BrokenPipeHandler for io::Result<()> {
    fn pipe_exit(self, device: &str) -> Result<()> {
        match self {
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => bail!(SilentExit { code: 0 }), // 直接啥都不返回
            result => result.with_context(|| format!("could not write to {}", device)),
        }
    }
}
