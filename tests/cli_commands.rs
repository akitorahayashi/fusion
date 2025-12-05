mod common;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn version_flag_works() {
    Command::cargo_bin("fusion").unwrap().arg("--version").assert().success();
}
