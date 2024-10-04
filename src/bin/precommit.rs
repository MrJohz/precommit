use std::{env::current_dir, io, process};

fn main() {
    let action = precommit::parse_args();
    let status = {
        let mut stdout = io::stdout().lock();
        let mut stderr = io::stderr().lock();
        precommit::run(
            action,
            &current_dir().expect("Could not access current working directory"),
            &mut stdout,
            &mut stderr,
        )
    };

    process::exit(status);
}
