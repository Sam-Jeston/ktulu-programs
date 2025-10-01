pub fn assert_logs_contain(logs: &[String], message: &str) {
    assert!(
        logs.iter().any(|l| l.contains(message)),
        "log does not contain message: {}",
        message
    );
}
