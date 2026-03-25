use std::collections::BTreeSet;

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
fn cleans_up_null_statements_in_empty_branches() {
    let rendered = render_source(
        "module top;\n  always_ff @(posedge clk) begin\n    if (rst) begin\n      ;\n      ;\n    end else begin\n      case (sel)\n        2'b11: begin\n          y <= a - b;\n          ;\n        end\n      endcase\n    end\n  end\nendmodule\n",
        &[],
        &BTreeSet::new(),
    )
    .unwrap();

    assert!(!rendered.contains("\n      ;\n      ;\n"));
    assert!(!rendered.contains("y <= a - b;\n          ;"));
}
