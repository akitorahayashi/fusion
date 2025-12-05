use assert_cmd::Command;

#[test]
fn version_flag_works() {
    Command::cargo_bin("fusion").unwrap().arg("--version").assert().success();
}
