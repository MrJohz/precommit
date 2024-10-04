use common::expect;

mod common;

#[test]
fn listing_contents_of_a_non_git_project_produces_an_error() {
    let _dir = common::tmpdir();

    exec!(self, "list").is_failure(101);
}

#[test]
fn listing_contents_of_an_empty_project_produces_no_output() {
    let _dir = common::tmpdir();

    exec!("git", "init").is_success();
    exec!(self, "list")
        .is_success()
        .stdout_equals(b"")
        .stderr_equals(
            b"Repository appears to be empty, some operations may have unexpected behaviour\n",
        );
}
