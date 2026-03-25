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

    let removable_id = parsed.candidates[0].id;

    let mut disabled = BTreeSet::new();
    disabled.insert(removable_id);

    let rendered = render_source(&parsed.source, &parsed.candidates, &disabled).unwrap();

    assert!(rendered.contains("module top;"));
    assert!(rendered.contains("logic a;"));
    assert_ne!(rendered, parsed.source);
}
