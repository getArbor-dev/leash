//! Binary smoke: help is exit 0, unknown command is exit 2.

#[test]
fn help_exits_zero() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_leash"))
        .arg("help")
        .output()
        .expect("run leash");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("leash session"));
    assert!(stdout.contains("leash ci"));
}

#[test]
fn unknown_command_exits_two() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_leash"))
        .arg("frobnicate")
        .output()
        .expect("run leash");
    assert_eq!(out.status.code(), Some(2));
}
