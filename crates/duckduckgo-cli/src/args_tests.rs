use crate::args::retry_from_args;

fn retry(args: &[&str]) -> Option<String> {
    retry_from_args(args.iter().map(|s| (*s).to_owned()))
}

#[test]
fn retry_from_args_accepts_space_and_equals_forms() {
    assert_eq!(retry(&["--retry", "5"]), Some("5".to_owned()));
    assert_eq!(retry(&["--retry=2"]), Some("2".to_owned()));
}

#[test]
fn retry_from_args_last_retry_or_no_retry_wins() {
    assert_eq!(retry(&["--no-retry", "--retry", "7"]), Some("7".to_owned()));
    assert_eq!(retry(&["--retry", "3", "--no-retry"]), Some("0".to_owned()));
}

#[test]
fn retry_from_args_ignores_other_arguments() {
    assert_eq!(retry(&["--num", "5", "rust"]), None);
}
