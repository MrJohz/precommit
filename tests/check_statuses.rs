use std::collections::HashSet;

mod common;

#[test]
fn check_raises_an_error_if_no_repository_exists() {
    let (_handle, dir) = common::dir();

    dir.exec_self(["check"]).is_failure(50).stdout_equals(b"");
}

#[test]
fn check_does_nothing_if_no_files_are_to_be_added() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.exec_self(["check", "-s", "false"])
        .is_success()
        .stdout_equals(b"");
}

#[test]
fn check_runs_check_command_for_each_file_to_be_added() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test1", "contents1");
    dir.git_add("test1");
    dir.file("test2", "contents2");
    dir.git_add("test2");

    let command = format!("echo 'check run' >> {:?}/output.log", dir.path());

    dir.exec_self(["check", "-s", &command]).is_success();

    let result = dir.read("output.log");
    assert_eq!(result, "check run\ncheck run\n");
}

#[test]
fn check_runs_check_command_in_working_directory_of_git_folder() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test", "contents");
    dir.git_add("test");

    let command = format!("echo $PWD > {:?}/output.log", dir.path());

    dir.exec_self(["check", "-s", &command]).is_success();

    let result = dir.read("output.log");
    assert_eq!(result, format!("{}\n", dir.path().to_string_lossy()));
}

#[test]
fn finds_correct_git_folder_when_run_in_subfolder_of_working_directory() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test", "contents");
    dir.git_add("test");

    let command = format!("echo $PWD > {:?}/output.log", dir.path());

    let subdir = dir.subdir("subdirectory");
    subdir.exec_self(["check", "-s", &command]).is_success();

    let result = dir.read("output.log");
    assert_eq!(result, format!("{}\n", dir.path().to_string_lossy()));
}

#[test]
fn passes_the_correct_file_name_to_the_command_being_run() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test", "contents");
    dir.git_add("test");

    dir.file("subdirectory/test", "contents");
    dir.git_add("subdirectory/test");

    let command = format!("echo {{}} >> {:?}/output.log", dir.path());

    let subdir = dir.subdir("subdirectory");
    subdir.exec_self(["check", "-s", &command]).is_success();

    let result = dir.read("output.log");
    assert_eq!(result.lines().collect::<HashSet<_>>(), {
        // use a set to handle the issue that order cannot be guaranteed
        let mut set = HashSet::new();
        set.insert("test");
        set.insert("subdirectory/test");
        set
    });
}

#[test]
fn the_command_name_replacement_string_can_be_changed_using_a_flag() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test", "contents");
    dir.git_add("test");

    let command = format!("echo flamingo >> {:?}/output.log", dir.path());

    let subdir = dir.subdir("subdirectory");
    subdir
        .exec_self(["check", "-Iflamingo", "-s", &command])
        .is_success();

    let result = dir.read("output.log");
    assert_eq!(result, "test\n");
}

#[test]
fn if_the_command_fails_the_command_output_is_shown_and_the_command_fails() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test", "contents");
    dir.git_add("test");

    let subdir = dir.subdir("subdirectory");
    subdir
        .exec_self(["check", "-s", ">&2 echo 'print this message'; false"])
        .is_failure(1)
        .stderr_contains("exit status: 1")
        .stderr_contains("print this message");
}

#[test]
fn if_multiple_commands_fail_all_the_command_outputs_are_shown() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test", "contents");
    dir.git_add("test");

    let subdir = dir.subdir("subdirectory");
    subdir
        .exec_self([
            "check",
            "-s",
            ">&2 echo 'message 1'; false",
            "-s",
            ">&2 echo 'message 2'; false",
        ])
        .is_failure(1)
        .stderr_contains("message 1")
        .stderr_contains("message 2");
}

#[test]
fn if_multiple_files_fail_all_the_command_outputs_are_shown() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test1", "contents");
    dir.git_add("test1");
    dir.file("test2", "contents");
    dir.git_add("test2");

    let subdir = dir.subdir("subdirectory");
    subdir
        .exec_self(["check", "-s", ">&2 echo 'error in file {}'; false"])
        .is_failure(1)
        .stderr_contains("error in file test1")
        .stderr_contains("error in file test2");
}

#[test]
fn stdout_is_never_printed_to_console() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test", "contents");
    dir.git_add("test");

    let subdir = dir.subdir("subdirectory");
    subdir
        .exec_self(["check", "-s", "echo 'print to stdout'; false"])
        .is_failure(1)
        .stderr_not_contains("print to stdout");
}

#[test]
fn check_status_is_fed_the_file_in_the_index() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test", "contents");
    dir.git_add("test");

    let command = format!("cat > {:?}/output.log", dir.path());

    dir.exec_self(["check", "-s", &command]).is_success();

    let result = dir.read("output.log");
    assert_eq!(result, "contents");
}
