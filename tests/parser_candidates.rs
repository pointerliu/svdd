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

#[test]
fn extracts_top_level_port_and_declaration_candidates() {
    let parsed = ParsedSource::parse_str(
        r#"module top (
  input wire clk,
  output reg y,
  output reg dummy_out
);
  logic a;
  assign a = clk;
endmodule
"#,
        "top.sv",
    )
    .unwrap();

    let spans: Vec<&str> = parsed
        .candidates
        .iter()
        .map(|candidate| &parsed.source[candidate.start..candidate.end])
        .collect();

    assert!(
        spans.iter().any(|span| span.contains("dummy_out")),
        "spans: {spans:#?}"
    );
    assert!(
        spans.iter().any(|span| span.contains("logic a;")),
        "spans: {spans:#?}"
    );
    assert!(
        spans.iter().any(|span| span.contains("assign a = clk;")),
        "spans: {spans:#?}"
    );
}

#[test]
fn extracts_statement_wrapper_candidates() {
    let parsed = ParsedSource::parse_str(
        r#"module top;
  logic a;
  always_ff @(posedge a) begin
    a <= 1'b1;
  end
endmodule
"#,
        "top.sv",
    )
    .unwrap();

    let spans: Vec<&str> = parsed
        .candidates
        .iter()
        .map(|candidate| &parsed.source[candidate.start..candidate.end])
        .collect();

    assert!(
        spans.iter().any(|span| span.contains("a <= 1'b1;")),
        "spans: {spans:#?}"
    );
}

#[test]
fn extracts_case_item_and_declaration_item_candidates() {
    let parsed = ParsedSource::parse_str(
        r#"module top;
  wire a, b, c;
  always_comb begin
    case (a)
      1'b0: b = c;
      default: c = b;
    endcase
  end
endmodule
"#,
        "top.sv",
    )
    .unwrap();

    let spans: Vec<&str> = parsed
        .candidates
        .iter()
        .map(|candidate| &parsed.source[candidate.start..candidate.end])
        .collect();

    assert!(
        spans.iter().any(|span| span.contains("b,")),
        "spans: {spans:#?}"
    );
    assert!(
        spans.iter().any(|span| span.contains("1'b0: b = c;")),
        "spans: {spans:#?}"
    );
}

#[test]
fn extracts_full_statement_candidate_not_just_assignment_body() {
    let parsed = ParsedSource::parse_str(
        r#"module top;
  always_ff @(posedge clk) begin
    if (rst) begin
      dummy_out <= 0;
    end
  end
endmodule
"#,
        "top.sv",
    )
    .unwrap();

    let spans: Vec<&str> = parsed
        .candidates
        .iter()
        .map(|candidate| &parsed.source[candidate.start..candidate.end])
        .collect();

    assert!(
        spans.iter().any(|span| span.contains("dummy_out <= 0;")),
        "spans: {spans:#?}"
    );
}
