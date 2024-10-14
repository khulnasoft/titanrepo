use assert_cmd::Command;

#[test]
fn test_find_correct_titan() {
    let mut cmd = Command::cargo_bin("titan").unwrap();
    cmd.assert()
        .append_context("titan", "no arguments")
        .append_context(
            "expect",
            "`titan` with no arguments should exit with code 1",
        )
        .code(1);

    let mut cmd = Command::cargo_bin("titan").unwrap();
    cmd.args(&["--cwd", "../examples/basic", "bin"])
        .assert()
        .append_context(
            "titan",
            "bin command with cwd flag set to package with local titan installed",
        )
        .append_context(
            "expect",
            "`titan --cwd ../../examples/basic bin` should print out local titan binary installed in examples/basic",
        )
        .success()
        .stdout(predicates::str::ends_with(
            "examples/basic/node_modules/.bin/titan\n",
        ));

    let mut cmd = Command::cargo_bin("titan").unwrap();
    cmd.args(&["--cwd", "..", "bin"])
        .assert()
        .append_context(
            "titan",
            "bin command with cwd flag set to package without local titan installed",
        )
        .append_context(
            "expect",
            "`titan --cwd .. bin` should print out current titan binary",
        )
        .success()
        .stdout(predicates::str::ends_with("target/debug/titan\n"));
}
