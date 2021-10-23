use std::env;
use std::io;
use std::process::Command;

// 将在包编译其他内容之前，被编译和调用，从而具备 Rust 代码所依赖的构建或生成的工件
fn main() {
    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap();
    let version = match env::var_os("PROFILE") {
        Some(profile) if profile == "release" => format!("v{}", pkg_version),
        _ => git_version().unwrap_or_else(|| format!("v{}-unknown", pkg_version)),
    };
    // 输出version
    println!("cargo:rustc-env=ZOXIDE_VERSION={}", version);

    // Since we are generating completions in the package directory, we need to
    // set this so that Cargo doesn't rebuild every time.
    //  Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=tests");

    // 用来生成相关的shell completions
    generate_completions().unwrap();
}

fn git_version() -> Option<String> {
    // 获取环境变量
    let dir = env::var("CARGO_MANIFEST_DIR").ok()?;
    // 输出相关的git
    let mut git = Command::new("git");
    // 输出版本
    // v0.7.4-9-g0019713-dirty
    git.args(&["-C", &dir, "describe", "--tags", "--broken"]);

    // 输出结果
    let output = git.output().ok()?;
    if !output.status.success() || output.stdout.is_empty() || !output.stderr.is_empty() {
        return None;
    }
    // 从标准输出中获取
    String::from_utf8(output.stdout).ok()
}

fn generate_completions() -> io::Result<()> {
    #[path = "src/app/_app.rs"]
    mod app;

    use app::App;
    use clap::IntoApp;
    use clap_generate::generate_to;
    use clap_generate::generators::{Bash, Elvish, Fish, PowerShell, Zsh};

    // 建立一个app的实例
    let app = &mut App::into_app();
    let bin_name = &env::var("CARGO_PKG_NAME").unwrap();
    let out_dir = "contrib/completions";

    // Generate a completions file for a specified shell at compile-time.
    generate_to::<Bash, _, _>(app, bin_name, out_dir)?;
    generate_to::<Elvish, _, _>(app, bin_name, out_dir)?;
    generate_to::<Fish, _, _>(app, bin_name, out_dir)?;
    generate_to::<PowerShell, _, _>(app, bin_name, out_dir)?;
    generate_to::<Zsh, _, _>(app, bin_name, out_dir)?;

    Ok(())
}
