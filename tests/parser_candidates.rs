use svdd::parser::ParsedSource;

#[test]
fn extracts_hierarchical_candidates_from_sv() {
    let parsed = ParsedSource::parse_str(
        r#"module top;
  logic a;
  always_comb begin
    if (a) begin
      a = 1'b1;
    end
  end
endmodule
"#,
        "top.sv",
    )
    .unwrap();

    assert!(!parsed.candidates.is_empty());
    assert_eq!(parsed.candidates[0].id, 0);
    assert!(parsed
        .candidates
        .iter()
        .any(|candidate| !candidate.children.is_empty()));
    assert!(parsed
        .candidates
        .iter()
        .any(|candidate| candidate.parent_id.is_some()));
    assert!(parsed
        .candidates
        .iter()
        .all(|candidate| !(candidate.start == 0 && candidate.end == parsed.source.len())));
    assert!(parsed.candidates.iter().all(|candidate| {
        let span = &parsed.source[candidate.start..candidate.end];
        !span.starts_with("module top")
    }));
}
