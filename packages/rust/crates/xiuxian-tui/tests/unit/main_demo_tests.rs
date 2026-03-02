use super::*;

#[test]
fn test_args_parsing_server() {
    let args = Args::parse_from(["xiuxian-tui", "--socket", "/test.sock", "--role", "server"]);
    assert_eq!(args.socket, "/test.sock");
    assert_eq!(args.role, "server");
}

#[test]
fn test_args_parsing_client() {
    let args = Args::parse_from([
        "xiuxian-tui",
        "--socket",
        "/test.sock",
        "--role",
        "client",
        "--pid",
        "1234",
    ]);
    assert_eq!(args.socket, "/test.sock");
    assert_eq!(args.role, "client");
    assert_eq!(args.pid, Some(1234));
}

#[test]
fn test_args_parsing_headless() {
    let args = Args::parse_from(["xiuxian-tui", "--socket", "/test.sock", "--headless"]);
    assert_eq!(args.socket, "/test.sock");
    assert!(args.headless);
}
