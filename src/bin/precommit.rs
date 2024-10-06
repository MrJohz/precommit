use std::{
    env::{args_os, current_dir},
    io, process,
};

fn main() {
    let action = precommit::parse_args(args_os());
    let status = {
        let stdout = io::stdout().lock();
        let stderr = io::stderr().lock();
        let world = precommit::WriterWorld::new(stdout, stderr);

        precommit::run(
            &current_dir().expect("Could not access current working directory"),
            action,
            &world,
        )
    };

    process::exit(status);
}
