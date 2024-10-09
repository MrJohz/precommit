# Precommit (to be renamed)

A tool for running linters, formatters, and other useful tools over your code before you commit it.

## Tasks

- [x] Help text and cli parsing error messages
- [x] Version flag
- [ ] Remove ByteStr dependency
- [x] Refactor `check.rs` and `run.rs` logic
- [ ] Add colours (and enable disabling colours)
- [ ] Convince subcommands to show colours if colours are enabled
- [x] Figure out how to run the correct shell
- [x] Improve formatting of stderr output in general
- [ ] Get tests running in CI
  - [ ] Broken pipe issues
  - [ ] MacOS directory changes
  - [ ] Windows ???
- [ ] Figure out how to put this into various packages
  - [ ] Cargo/crates/binstall?
  - [ ] NPM (via GitHub releases?)
  - [ ] Homebrew
- [ ] Formatting
  - [ ] Implement formatting of object files
  - [ ] Implement patch logic from https://github.com/hallettj/git-format-staged/
  - [ ] Add tests for formatting
