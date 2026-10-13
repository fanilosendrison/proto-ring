use num_bigint::BigUint;

use super::PortablePattern;
use super::model::Node;

#[test]
fn deeply_nested_noncapturing_groups_use_explicit_stacks() {
    let depth = 2_048;
    let source = format!("{}a{}", "(?:".repeat(depth), ")".repeat(depth));
    let pattern = PortablePattern::compile(&source).expect("deep pattern compiles");
    assert!(pattern.full_match("a").expect("matcher succeeds").is_some());
}

#[test]
fn quantifier_above_u64_is_stored_exactly() {
    let decimal = "18446744073709551616";
    let pattern = PortablePattern::compile(&format!("a{{{decimal}}}"))
        .expect("arbitrary integer bound compiles");
    let minimum = pattern
        .program
        .nodes
        .iter()
        .find_map(|node| match node {
            Node::Repeat { minimum, .. } => Some(minimum),
            _ => None,
        })
        .expect("repeat exists");
    assert_eq!(minimum.to_string(), decimal);
    assert!(minimum > &BigUint::from(u64::MAX));
    assert!(pattern.full_match("a").expect("matcher succeeds").is_none());
}

#[test]
fn canonical_4301_digit_bound_is_accepted_and_pruned() {
    let decimal = format!("1{}", "0".repeat(4_300));
    let pattern =
        PortablePattern::compile(&format!("a{{{decimal}}}")).expect("4301-digit bound compiles");
    assert!(pattern.full_match("a").expect("matcher succeeds").is_none());
}

#[test]
fn bounded_nullable_repeat_uses_heap_execution_state() {
    let pattern = PortablePattern::compile("(a?){1500}").expect("bounded repeat compiles");
    let matched = pattern
        .full_match("")
        .expect("matcher succeeds")
        .expect("empty input matches");
    assert_eq!(matched.positional_capture(1), Some(""));
}

#[test]
fn absent_capture_differs_from_empty_participation() {
    let pattern = PortablePattern::compile("(a)?()b").expect("pattern compiles");
    let matched = pattern
        .full_match("b")
        .expect("matcher succeeds")
        .expect("input matches");
    assert_eq!(matched.positional_capture(1), None);
    assert_eq!(matched.positional_capture(2), Some(""));
}

#[test]
fn named_capture_aliases_positional_state() {
    let pattern = PortablePattern::compile("(?P<name>a)").expect("pattern compiles");
    let matched = pattern
        .full_match("a")
        .expect("matcher succeeds")
        .expect("input matches");
    assert_eq!(matched.positional_capture(1), Some("a"));
    assert_eq!(matched.named_capture("name"), Some("a"));
}

#[test]
fn later_omission_retains_nested_capture() {
    let pattern = PortablePattern::compile("((a)?){2}").expect("pattern compiles");
    let matched = pattern
        .full_match("a")
        .expect("matcher succeeds")
        .expect("input matches");
    assert_eq!(matched.positional_capture(1), Some(""));
    assert_eq!(matched.positional_capture(2), Some("a"));
}

#[test]
fn unsupported_grammar_fails_closed() {
    for source in [".", "\\d", "(?=a)", "[a-b-c]", "()*", "a{01}", "a**"] {
        assert!(
            PortablePattern::compile(source).is_err(),
            "accepted {source}"
        );
    }
}
