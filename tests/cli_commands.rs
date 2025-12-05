use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn version_flag_works() {
    Command::cargo_bin("fusion")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}
