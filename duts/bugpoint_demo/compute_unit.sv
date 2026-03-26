module compute_unit #(
    parameter WIDTH = 8
) (
    input wire clk,
    input wire rst,
    input wire [WIDTH-1:0] a,
    input wire [WIDTH-1:0] b,
    input wire [1:0] sel,
    output reg [WIDTH-1:0] y,
    output reg [WIDTH-1:0] dummy_out
);

  wire [WIDTH-1:0] dummy_wire_1;
  wire [WIDTH-1:0] dummy_wire_2;
  wire [WIDTH-1:0] dummy_wire_3;
  wire [WIDTH-1:0] dummy_wire_4;
  wire [WIDTH-1:0] dummy_wire_5;
  wire [WIDTH-1:0] dummy_wire_6;
  wire [WIDTH-1:0] intermediate_1;
  wire [WIDTH-1:0] intermediate_2;
  wire [WIDTH-1:0] intermediate_3;

  reg [WIDTH-1:0] dummy_reg_1;
  reg [WIDTH-1:0] dummy_reg_2;
  reg [WIDTH-1:0] dummy_reg_3;
  reg [WIDTH-1:0] dummy_reg_4;
  reg [WIDTH-1:0] dummy_reg_5;
  reg [WIDTH-1:0] dummy_reg_6;
  reg [WIDTH-1:0] dummy_reg_7;
  reg [WIDTH-1:0] dummy_reg_8;
  reg [WIDTH-1:0] stored_value_1;
  reg [WIDTH-1:0] stored_value_2;
  reg [WIDTH-1:0] stored_value_3;

  assign dummy_wire_1 = a & 8'hFF;
  assign dummy_wire_2 = b | 8'h00;
  assign dummy_wire_3 = a ^ 8'h55;
  assign dummy_wire_4 = b ^ 8'hAA;
  assign dummy_wire_5 = ~a;
  assign dummy_wire_6 = ~b;
  assign intermediate_1 = dummy_wire_1 + dummy_wire_2;
  assign intermediate_2 = dummy_wire_3 - dummy_wire_4;
  assign intermediate_3 = intermediate_1 ^ intermediate_2;

  always_comb begin
    dummy_reg_1 = a + 1;
    dummy_reg_2 = b + 1;
    dummy_reg_3 = a - 1;
    dummy_reg_4 = b - 1;
    dummy_reg_5 = a & b;
    dummy_reg_6 = a | b;
    dummy_reg_7 = a ^ b;
    dummy_reg_8 = ~a;
    stored_value_1 = dummy_reg_1;
    stored_value_2 = dummy_reg_2;
    stored_value_3 = dummy_reg_3;
  end

  always_ff @(posedge clk) begin
    if (rst) begin
      y <= 0;
      dummy_out <= 0;
    end else begin
      case (sel)
        2'b00: begin
          y <= a + b;
          dummy_out <= dummy_wire_1;
        end
        2'b01: begin
          y <= a - b;
          dummy_out <= dummy_wire_2;
        end
        2'b10: begin
          y <= a & b;
          dummy_out <= dummy_wire_3;
        end
        2'b11: begin
          y <= a - b;
          dummy_out <= dummy_wire_4;
        end
        default: begin
          y <= 0;
          dummy_out <= 0;
        end
      endcase
    end
  end

  wire [WIDTH-1:0] never_used_1;
  wire [WIDTH-1:0] never_used_2;
  wire [WIDTH-1:0] never_used_3;
  wire [WIDTH-1:0] never_used_4;
  wire [WIDTH-1:0] never_used_5;

  assign never_used_1 = {WIDTH{1'b0}};
  assign never_used_2 = {WIDTH{1'b1}};
  assign never_used_3 = {{(WIDTH/2){1'b0}}, a[WIDTH-1:WIDTH/2]};
  assign never_used_4 = {{(WIDTH/2){1'b0}}, b[WIDTH/2-1:0]};
  assign never_used_5 = a + b + 1;

  reg [WIDTH-1:0] padding_reg_1;
  reg [WIDTH-1:0] padding_reg_2;
  reg [WIDTH-1:0] padding_reg_3;
  reg [WIDTH-1:0] padding_reg_4;

  always_comb begin
    padding_reg_1 = 8'h00;
    padding_reg_2 = 8'hFF;
    padding_reg_3 = a ^ b;
    padding_reg_4 = ~(a | b);
  end

endmodule
