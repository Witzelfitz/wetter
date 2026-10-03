use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_wetter"))
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("start wetter")
}

#[test]
fn version_matches_package_and_needs_no_network() {
    for flag in ["--version", "-V"] {
        let output = run(&[flag]);
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap().trim(),
            format!("wetter {}", env!("CARGO_PKG_VERSION"))
        );
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn help_documents_public_modes() {
    let output = run(&["--help"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    for flag in ["--tui", "--plain", "--json", "--version", "--pick"] {
        assert!(text.contains(flag), "missing {flag}");
    }
}

#[test]
fn invalid_usage_returns_failure_without_polluting_stdout() {
    for args in [
        vec!["--invalid"],
        vec!["--plain", "--json"],
        vec!["--pick", "0"],
        vec!["--json"],
        vec!["--tui"],
    ] {
        let output = run(&args);
        assert!(!output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert!(!output.stderr.is_empty(), "{args:?}");
    }
}
