use super::*;
use lance::deps::arrow_array::Array;

#[test]
fn lowercase_string_array_roundtrip() {
    let arr = StringArray::from(vec!["Git.Commit", "Writer.Polish", "KNOWLEDGE.RECALL"]);
    let lower = lowercase_string_array(&arr);
    assert_eq!(lower.len(), 3);
    assert_eq!(lower.value(0), "git.commit");
    assert_eq!(lower.value(1), "writer.polish");
    assert_eq!(lower.value(2), "knowledge.recall");
}

#[test]
fn build_name_lower_arrow_indexed() {
    let keys = ["Git.Commit", "Writer.Polish", "knowledge.recall"];
    let (keys_ordered, names_lower) = build_name_lower_arrow(keys.iter());
    assert_eq!(keys_ordered.len(), 3);
    assert_eq!(names_lower.value(0), "git.commit");
    assert_eq!(names_lower.value(1), "writer.polish");
    assert_eq!(names_lower.value(2), "knowledge.recall");
    assert_eq!(keys_ordered[0], "Git.Commit");
}

#[test]
fn automaton_with_phrase_single_word_exact() {
    let query_parts = ["commit"];
    let full = "commit";
    let Some((ac, exact_id)) = build_name_token_automaton_with_phrase(&query_parts, full) else {
        panic!("expected automaton for non-empty query")
    };
    assert_eq!(exact_id, Some(PatternID::ZERO));

    let r = count_name_token_matches_and_exact(&ac, "git_commit", exact_id);
    assert!(r.exact_phrase, "commit is substring of git_commit");
    assert_eq!(
        r.token_count, 0,
        "single-word query is only pattern 0, not counted as token"
    );

    let r2 = count_name_token_matches_and_exact(&ac, "git_status", exact_id);
    assert_eq!(r2.token_count, 0);
    assert!(!r2.exact_phrase);
}

#[test]
fn automaton_with_phrase_multi_word() {
    let query_parts = ["git", "commit"];
    let full = "git commit";
    let Some((ac, exact_id)) = build_name_token_automaton_with_phrase(&query_parts, full) else {
        panic!("expected automaton for valid multi-word query")
    };
    assert_eq!(exact_id, Some(PatternID::ZERO));

    let r = count_name_token_matches_and_exact(&ac, "git_commit", exact_id);
    assert!(r.token_count >= 1, "token match for git and commit");
    assert!(
        !r.exact_phrase,
        "git_commit does not contain substring 'git commit'"
    );
}
