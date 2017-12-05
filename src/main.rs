extern crate cargo;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use structopt::StructOpt;
use cargo::util::important_paths;
use cargo::util::config::Config;
use cargo::ops::{self, CompileOptions, CompileMode};
use cargo::core::Workspace;
use cargo::core::shell::Verbosity;

use std::env;
use std::path::PathBuf;
use std::process::Command;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "cargo-wasm", about = "A Cargo subcommand to make working with WASM easier.")]
struct Opt {
    #[structopt(long = "release", help = "Compile using release mode")]
    release: bool,

    #[structopt(short = "v", long = "verbose", help = "Compile with verbose output")]
    verbose: bool,
}

fn main() {
    let opt = Opt::from_args();
    compile(opt);
}

fn compile(opt: Opt) {
    let crate_manifest = &important_paths::find_root_manifest_for_wd(
        None,
        &env::current_dir().expect("Could not access current working directory."),
    ).expect(
        "Could not find a Rust project in this directory or any of the parent directories.",
    );

    let config = &Config::default().expect("Could not create default Config struct.");
    config.shell().set_verbosity(if opt.verbose {
        Verbosity::Verbose
    } else {
        Verbosity::Normal
    });

    let workspace =
        &Workspace::new(crate_manifest, config).expect("Could not create Workspace struct.");

    let compile_options = &mut CompileOptions::default(config, CompileMode::Build);
    compile_options.target = Some("wasm32-unknown-unknown");
    compile_options.release = opt.release;

    let compilation = if let Ok(res) = ops::compile(workspace, compile_options) {
        res
    } else {
        return;
    };

    let crate_dir = &mut crate_manifest.clone();
    crate_dir.pop();
    for (pkg, outputs) in compilation.libraries {
        for (_, wasm) in outputs {
            post_process(crate_dir, pkg.name(), &wasm);
        }
    }
}

fn post_process(crate_dir: &PathBuf, pkg_name: &str, wasm: &PathBuf) {
    let output = &crate_dir.join(pkg_name.to_string() + ".wasm");
    Command::new("wasm-gc")
        .arg(wasm)
        .arg(output)
        .status()
        .expect("wasm-gc command failed");
    println!("wasm-gc processed: {}.wasm", pkg_name);
}
