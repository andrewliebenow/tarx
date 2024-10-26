use std::env;
use tracing_subscriber::fmt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<(), i32> {
    // TODO
    env::set_var("RUST_BACKTRACE", "1");

    tracing_subscriber::registry()
        .with(fmt::layer().pretty())
        .init();

    let result = start();

    if let Err(er) = result {
        tracing::error!(
            backtrace = %er.backtrace(),
            error = %er,
        );

        return Err(1_i32);
    }

    Ok(())
}

// TODO
// Only recompile foreign code when it changes
#[expect(clippy::unnecessary_wraps, reason = "Conditional compilation")]
fn start() -> anyhow::Result<()> {
    #[cfg(feature = "foreign")]
    {
        use std::{
            io::{self, Write},
            process::Command,
            str,
        };

        const LIBRARY_NAME: &str = "foreign";

        let library_name_with_lib_prefix = format!("lib{LIBRARY_NAME}");

        let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
        let out_dir = env::var("OUT_DIR")?;

        let foreign_path = format!("{cargo_manifest_dir}/foreign");
        let out_dir_unique_path = format!("{out_dir}/zfa935eb0aa61a49472ec9753595ad826");

        let output = Command::new("go")
            .args([
                "build",
                // "If used, this flag must be the first one in the command line."
                &format!("-C={foreign_path}"),
                "-buildmode=c-archive",
                &format!("-o={out_dir_unique_path}/{library_name_with_lib_prefix}.a"),
                "--",
                &format!("{foreign_path}/main.go"),
            ])
            .output()?;

        let stderr = output.stderr;
        let stdout = output.stdout;

        let stderr_str_result = str::from_utf8(&stderr);
        let stdout_str_result = str::from_utf8(&stdout);

        tracing::info!(
            ?stderr_str_result,
            ?stdout_str_result,
            "Output from spawned process"
        );

        anyhow::ensure!(output.status.success(), "Failed to compile foreign code");

        let bindings = bindgen::Builder::default()
            .header(format!(
                "{out_dir_unique_path}/{library_name_with_lib_prefix}.h"
            ))
            .generate()?;

        bindings.write_to_file(format!("{out_dir}/{library_name_with_lib_prefix}.rs"))?;

        {
            let mut stdout_lock = io::stdout().lock();

            stdout_lock
                .write_all(format!("cargo:rustc-link-lib=static={LIBRARY_NAME}\n").as_bytes())?;
            stdout_lock.write_all(
                format!("cargo:rustc-link-search=native={out_dir_unique_path}\n").as_bytes(),
            )?;

            drop(stdout_lock);
        }

        Ok(())
    }

    #[cfg(not(feature = "foreign"))]
    {
        Ok(())
    }
}
