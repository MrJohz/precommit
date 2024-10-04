mod common;

#[test]
fn listing_contents_of_a_non_git_project_produces_an_error() {
    let dir = common::dir();

    dir.exec_self(["list"]).is_failure(50);
}

#[test]
fn listing_contents_of_an_empty_project_produces_no_output() {
    let dir = common::dir();

    dir.git_init();
    dir.exec_self(["list"])
        .is_success()
        .stdout_equals(b"")
        .stderr_equals(b"");
}

#[test]
fn listing_contents_of_a_project_with_no_commits_and_unstaged_files_produces_no_output() {
    let dir = common::dir();

    dir.git_init();
    dir.file("test", "assorted file contents");

    dir.exec_self(["list"]).is_success().stdout_equals(b"");
}

#[test]
fn listing_contents_of_a_project_with_no_commits_and_staged_files_lists_only_staged_files() {
    let dir = common::dir();

    dir.git_init();
    dir.file("test", "assorted file contents");
    dir.file("test2", "other file contents");
    dir.git_add("test");

    dir.exec_self(["list"])
        .is_success()
        .stdout_equals(b"test\n");
}

#[test]
fn lists_all_files_even_from_files_in_deep_folders() {
    let dir = common::dir();

    dir.git_init();

    dir.file("nested/deep/within/test", "assorted file contents");
    dir.file("nested/deep/within/test2", "other file contents");
    dir.git_add("nested/deep/within/test");
    dir.git_add("nested/deep/within/test2");

    dir.exec_self(["list"])
        .is_success()
        .stdout_equals(b"nested/deep/within/test\nnested/deep/within/test2\n");
}

#[test]
fn files_added_before_a_commit_are_not_listed() {
    let dir = common::dir();

    dir.git_init();

    dir.file("test", "assorted file contents");
    dir.file("test2", "other file contents");
    dir.git_add("test");
    dir.git_add("test2");

    dir.git_commit();

    dir.exec_self(["list"]).is_success().stdout_equals(b"");
}

#[test]
fn files_added_after_a_commit_are_listed() {
    let dir = common::dir();

    dir.git_init();

    dir.git_commit();

    dir.file("test", "assorted file contents");
    dir.file("test2", "other file contents");
    dir.git_add("test");
    dir.git_add("test2");

    dir.exec_self(["list"])
        .is_success()
        .stdout_equals(b"test\ntest2\n");
}
