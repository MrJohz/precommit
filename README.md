# Precommit (to be renamed)

A tool for running linters, formatters, and other useful tools over your code before you commit it.

## Tasks

- [ ] Implement formatting of object files
- [ ] Implement patch logic from https://github.com/hallettj/git-format-staged/
- [ ] Add tests for formatting
- [ ] Help text and cli parsing error messages
- [ ] Refactor `check.rs` and `run.rs` logic
- [ ] Add colours (and enable disabling colours)
- [ ] Convince subcommands to show colours if colours are enabled
- [ ] Improve formatting of stderr output in general
- [ ] Figure out how to put this into various packages
  - [ ] Cargo/crates/binstall?
  - [ ] NPM (via GitHub releases?)
  - [ ] Homebrew
