use std::collections::BTreeSet;

use svdd::model::CandidateKind;
use svdd::render::render_source;

#[test]
fn removes_disabled_candidate_spans_without_touching_other_text() {
    let parsed = svdd::parser::ParsedSource::parse_str(
        r#"module top;
  logic a;
  always_comb begin
    if (a) begin
      a = 1'b0;
    end
  end
endmodule
"#,
        "top.sv",
    )
    .unwrap();

    let removable_id = parsed
        .candidates
        .iter()
        .find(|candidate| {
            let span = &parsed.source[candidate.start..candidate.end];
            span.contains("a = 1'b0;")
        })
        .map(|candidate| candidate.id)
        .unwrap();

    let mut disabled = BTreeSet::new();
    disabled.insert(removable_id);

    let rendered = render_source(&parsed.source, &parsed.candidates, &disabled).unwrap();

    assert!(rendered.contains("module top;"));
    assert!(rendered.contains("logic a;"));
    assert_ne!(rendered, parsed.source);
}

#[test]
fn removes_statement_terminator_with_nonblocking_assignment_candidate() {
    let parsed = svdd::parser::ParsedSource::parse_str(
        r#"module top;
  logic a;
  always_ff @(posedge a) begin
    a <= 1'b0;
  end
endmodule
"#,
        "top.sv",
    )
    .unwrap();

    let removable_id = parsed
        .candidates
        .iter()
        .find(|candidate| {
            let span = &parsed.source[candidate.start..candidate.end];
            span.contains("a <= 1'b0")
        })
        .map(|candidate| candidate.id)
        .unwrap();

    let mut disabled = BTreeSet::new();
    disabled.insert(removable_id);

    let rendered = render_source(&parsed.source, &parsed.candidates, &disabled).unwrap();

    assert!(!rendered.contains("a <= 1'b0;"));
    assert!(!rendered.contains("begin\n    ;\n  end"));
}

#[test]
fn removing_reset_statement_does_not_leave_bare_semicolon() {
    let parsed = svdd::parser::ParsedSource::parse_str(
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

    let removable_id = parsed
        .candidates
        .iter()
        .find(|candidate| {
            let span = &parsed.source[candidate.start..candidate.end];
            span.contains("dummy_out <= 0;")
        })
        .map(|candidate| candidate.id)
        .unwrap();

    let mut disabled = BTreeSet::new();
    disabled.insert(removable_id);

    let rendered = render_source(&parsed.source, &parsed.candidates, &disabled).unwrap();

    assert!(!rendered.contains("dummy_out <= 0;"));
    assert!(!rendered.contains("\n      ;\n"));
}

#[test]
fn removes_declaration_item_from_comma_separated_list() {
    let parsed = svdd::parser::ParsedSource::parse_str(
        r#"module top;
  wire a, b, c;
endmodule
"#,
        "top.sv",
    )
    .unwrap();

    let removable_id = parsed
        .candidates
        .iter()
        .find(|candidate| {
            let span = &parsed.source[candidate.start..candidate.end];
            span.contains("b") && !span.contains("wire")
        })
        .map(|candidate| candidate.id)
        .unwrap();

    let mut disabled = BTreeSet::new();
    disabled.insert(removable_id);

    let rendered = render_source(&parsed.source, &parsed.candidates, &disabled).unwrap();

    assert!(rendered.contains("wire a, c;"));
    assert!(!rendered.contains("wire a, , c;"));
    assert!(!rendered.contains("b"));
}

#[test]
fn removes_whole_declaration_when_last_item_is_removed() {
    let parsed = svdd::parser::ParsedSource::parse_str(
        r#"module top;
  wire lone_wire;
  reg lone_reg;
endmodule
"#,
        "top.sv",
    )
    .unwrap();

    let remove_wire = parsed
        .candidates
        .iter()
        .find(|candidate| parsed.source[candidate.start..candidate.end].contains("lone_wire"))
        .map(|candidate| candidate.id)
        .unwrap();
    let remove_reg = parsed
        .candidates
        .iter()
        .find(|candidate| parsed.source[candidate.start..candidate.end].contains("lone_reg"))
        .map(|candidate| candidate.id)
        .unwrap();

    let mut disabled = BTreeSet::new();
    disabled.insert(remove_wire);
    disabled.insert(remove_reg);

    let rendered = render_source(&parsed.source, &parsed.candidates, &disabled).unwrap();

    assert!(!rendered.contains("wire lone_wire;"));
    assert!(!rendered.contains("reg lone_reg;"));
    assert!(!rendered.contains("wire ;"));
    assert!(!rendered.contains("reg ;"));
}

#[test]
fn removes_case_item_while_preserving_case_syntax() {
    let parsed = svdd::parser::ParsedSource::parse_str(
        r#"module top;
  always_comb begin
    case (sel)
      2'b00: a = b;
      default: a = c;
    endcase
  end
endmodule
"#,
        "top.sv",
    )
    .unwrap();

    let removable_id = parsed
        .candidates
        .iter()
        .find(|candidate| {
            candidate.kind == CandidateKind::CaseItem && {
                let span = &parsed.source[candidate.start..candidate.end];
                span.contains("2'b00: a = b;")
            }
        })
        .map(|candidate| candidate.id)
        .unwrap();

    let mut disabled = BTreeSet::new();
    disabled.insert(removable_id);

    let rendered = render_source(&parsed.source, &parsed.candidates, &disabled).unwrap();

    assert!(rendered.contains("case (sel)"));
    assert!(rendered.contains("default: a = c;"));
    assert!(!rendered.contains("2'b00: a = b;"));
    assert!(rendered.contains("endcase"));
}
