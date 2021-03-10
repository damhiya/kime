pub use daemonize;
pub use kime_log;
pub use pico_args;
pub use xdg;

#[doc(hidden)]
pub mod build {
    include!(concat!(env!("OUT_DIR"), "/shadow.rs"));
}

#[macro_export]
macro_rules! cli_boilerplate {
    ($($help:expr,)*) => {{
        let mut args = $crate::pico_args::Arguments::from_env();

        if args.contains(["-h", "--help"]) {
            println!("-h or --help: show help");
            println!("-v or --version: show version");
            println!("--verbose: show verbose log");
            $(
                println!($help);
            )*
            return;
        }

        if args.contains(["-v", "--version"]) {
            $crate::print_version!();
            return;
        }

        let level = if args.contains("--verbose") {
            $crate::kime_log::LevelFilter::Trace
        } else {
            $crate::kime_log::LevelFilter::Info
        };

        $crate::kime_log::enable_logger(level);

        log::info!(
            "Start {}: {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        args
    }};
}

#[macro_export]
macro_rules! daemon_boilerplate {
    ($($help:expr,)*) => {{
        let mut args = $crate::cli_boilerplate!(
            "-d: run as normal program",
            "-D: run as daemon(default)",
            $($help,)*
        );

        let run_daemon = if args.contains("-D") {
            false
        } else {
            true
        };

        if run_daemon {
            let pid = $crate::xdg::BaseDirectories::new().ok().and_then(|dir|
                dir.place_runtime_file(concat!(env!("CARGO_PKG_NAME"), ".pid")).ok()
            ).unwrap_or(concat!("/tmp/", env!("CARGO_PKG_NAME"), ".pid").into());

            use $crate::daemonize::DaemonizeError::*;

            match $crate::daemonize::Daemonize::new()
                .pid_file(pid)
                .working_directory("/tmp")
                .start() {
                    Ok(_) => Ok(()),
                    Err(LockPidfile(_)) => {
                        log::warn!("Already running");
                        ::std::process::exit(0);
                    }
                    err => err,
                }
        } else {
            Ok(())
        }
    }};
}

#[macro_export]
macro_rules! print_version {
    () => {
        if $crate::build::TAG.is_empty() {
            println!(
                "kime(git) {} {}",
                $crate::build::COMMIT_DATE,
                $crate::build::SHORT_COMMIT
            );
        } else {
            println!("kime(release) {}", $crate::build::TAG);
        }
        println!("`{}` {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
