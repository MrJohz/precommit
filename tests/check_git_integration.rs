mod common;

#[test]
fn check_passes_file_contents_to_file_stdin() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("test", "contents");
    dir.git_add("test");

    let command = format!("cat > {:?}/output.log", dir.path());

    dir.exec_self(["check", "-s", &command]).is_success();

    let result = dir.read("output.log");
    assert_eq!(result, "contents");
}

#[test]
fn check_ignores_files_that_are_not_in_index() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("added", "contents of added file\n");
    dir.git_add("added");

    dir.file("dirty", "this contents is not present\n");

    let command = format!("cat >> {:?}/output.log", dir.path());

    dir.exec_self(["check", "-s", &command]).is_success();

    let result = dir.read("output.log");
    assert_eq!(result, "contents of added file\n");
}

#[test]
fn check_passes_contents_of_file_as_seen_in_index() {
    let (_handle, dir) = common::dir();

    dir.git_init();

    dir.file("file", "contents as seen in the index\n");
    dir.git_add("file");

    dir.file("file", "changed in working directory\n");

    let command = format!("cat >> {:?}/output.log", dir.path());

    dir.exec_self(["check", "-s", &command]).is_success();

    let result = dir.read("output.log");
    assert_eq!(result, "contents as seen in the index\n");
}
