mod common;

#[test]
fn check_diffs_command_output_against_existing_contents() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test", "contents");
    dir.git_add("test");

    let command = format!("tee {:?}/output.log", dir.path());

    dir.exec_self(["check", "-d", &command]).is_success();

    let result = dir.read("output.log");
    assert_eq!(result, "contents");
}

#[test]
fn check_diffs_errors_if_diff_is_not_identical() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test.txt", "contents");
    dir.git_add("test.txt");

    dir.exec_self(["check", "-d", "echo 'hello'"])
        .is_failure(1)
        .stderr_contains("test.txt");
}

#[test]
fn diff_commands_can_emit_to_stderr_without_failing() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test.txt", "contents");
    dir.git_add("test.txt");

    dir.exec_self(["check", "-d", "cat; >&2 echo 'this is spurious text'"])
        .is_success();
}

#[test]
fn stderr_from_diff_commands_is_shown_to_user_when_diff_command_fails() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test.txt", "contents");
    dir.git_add("test.txt");

    dir.exec_self([
        "check",
        "-d",
        "echo 'this is sent to stdout'; >&2 echo 'this is spurious text'",
    ])
    .is_failure(1)
    .stderr_contains("this is spurious text")
    .stderr_not_contains("this is sent to stdout");
}

#[test]
fn diff_commands_fail_if_the_command_returns_nonzero() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test.txt", "contents");
    dir.git_add("test.txt");

    dir.exec_self(["check", "-d", "false"])
        .is_failure(1)
        .stderr_contains("exit status: 1");
}
