#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::map_unwrap_or,
    clippy::option_as_ref_deref,
    clippy::unreadable_literal,
    clippy::useless_conversion,
    clippy::match_wildcard_for_single_variants,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::manual_async_fn,
    clippy::manual_let_else,
    clippy::manual_assert,
    clippy::manual_string_new,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::similar_names,
    clippy::format_collect,
    clippy::async_yields_async,
    clippy::assigning_clones
)]

use omni_agent::{parse_crawl_shortcut, parse_graph_bridge_shortcut};

#[test]
fn parse_crawl_shortcut_accepts_basic_url_defaults() {
    let shortcut = parse_crawl_shortcut("crawl https://example.com/").expect("parsed");
    assert_eq!(shortcut.url, "https://example.com/");
    assert!(shortcut.fit_markdown);
    assert_eq!(shortcut.max_depth, 1);
    assert!(shortcut.action.is_none());
    assert!(!shortcut.return_skeleton);
}

#[test]
fn parse_crawl_shortcut_accepts_wrapped_url() {
    let shortcut = parse_crawl_shortcut("crawl `https://example.com/path?q=1`").expect("parsed");
    assert_eq!(shortcut.url, "https://example.com/path?q=1");
}

#[test]
fn parse_crawl_shortcut_accepts_options() {
    let shortcut = parse_crawl_shortcut(
        "crawl https://example.com --depth 2 --raw --no-fit-markdown --return-skeleton",
    )
    .expect("parsed");
    assert_eq!(shortcut.url, "https://example.com");
    assert!(!shortcut.fit_markdown);
    assert_eq!(shortcut.max_depth, 2);
    assert_eq!(shortcut.action.as_deref(), Some("crawl"));
    assert!(shortcut.return_skeleton);
}

#[test]
fn parse_crawl_shortcut_rejects_non_command_text() {
    assert!(parse_crawl_shortcut("please crawl https://example.com").is_none());
    assert!(parse_crawl_shortcut("crawl").is_none());
}

#[test]
fn parse_crawl_shortcut_rejects_extra_tokens_and_invalid_options() {
    assert!(parse_crawl_shortcut("crawl https://example.com summarize it").is_none());
    assert!(parse_crawl_shortcut("crawl https://example.com --unknown").is_none());
    assert!(parse_crawl_shortcut("crawl https://example.com --depth nope").is_none());
}

#[test]
fn crawl_shortcut_args_match_defaults() {
    let parsed = parse_crawl_shortcut("crawl https://example.com").expect("parsed");
    let args = parsed.to_arguments();
    assert_eq!(args["url"], "https://example.com");
    assert_eq!(args["fit_markdown"], true);
    assert_eq!(args["max_depth"], 1);
    assert!(args.get("action").is_none());
    assert!(args.get("return_skeleton").is_none());
}

#[test]
fn crawl_shortcut_args_include_optional_fields() {
    let parsed =
        parse_crawl_shortcut("crawl https://example.com --skeleton --return-skeleton --depth 3")
            .expect("parsed");
    let args = parsed.to_arguments();
    assert_eq!(args["action"], "skeleton");
    assert_eq!(args["return_skeleton"], true);
    assert_eq!(args["max_depth"], 3);
}

#[test]
fn parse_graph_bridge_shortcut_accepts_tool_only() {
    let shortcut = parse_graph_bridge_shortcut("graph researcher.run_research_graph")
        .expect("parsed graph bridge shortcut");
    assert_eq!(shortcut.tool_name, "researcher.run_research_graph");
    assert!(shortcut.arguments.is_none());
}

#[test]
fn parse_graph_bridge_shortcut_accepts_tool_with_json_object_args() {
    let shortcut = parse_graph_bridge_shortcut(
        r#"graph researcher.run_research_graph {"repo_url":"https://github.com/example/repo","mode":"full"}"#,
    )
    .expect("parsed graph bridge shortcut with args");
    assert_eq!(shortcut.tool_name, "researcher.run_research_graph");
    assert_eq!(
        shortcut.arguments.expect("args object")["repo_url"],
        "https://github.com/example/repo"
    );
}

#[test]
fn parse_graph_bridge_shortcut_rejects_invalid_forms() {
    assert!(parse_graph_bridge_shortcut("graph").is_none());
    assert!(parse_graph_bridge_shortcut("graph tool [1,2,3]").is_none());
    assert!(parse_graph_bridge_shortcut("graph tool {invalid json}").is_none());
    assert!(parse_graph_bridge_shortcut("please graph tool {}").is_none());
}
