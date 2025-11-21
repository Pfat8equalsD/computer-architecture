module top(input wire [1:0] clock_reset, input wire [17:0] i, output wire [15:0] o);
   wire [31:0] od;
   wire [15:0] d;
   wire [15:0] q;
   assign o = od[15:0];
   top_memory c0(.clock_reset(clock_reset), .i(d[15:0]), .o(q[15:0]));
   assign d = od[31:16];
   assign od = kernel_register_kernel(clock_reset, i, q);
   function [31:0] kernel_register_kernel(input reg [1:0] arg_0, input reg [17:0] arg_1, input reg [15:0] arg_2);
         reg [0:0] or0;
         reg [17:0] or1;
         reg [0:0] or2;
         reg [0:0] or3;
         reg [0:0] or4;
         reg [0:0] or5;
         reg [15:0] or6;
         reg [15:0] or7;
         reg [0:0] or8;
         reg [0:0] or9;
         reg [15:0] or10;
         reg [15:0] or11;
         reg [15:0] or12;
         reg [31:0] or13;
         reg [1:0] or14;
         localparam ol0 = 1'b1;
         localparam ol1 = 1'b0;
         localparam ol2 = 16'b0000000000000000;
         localparam ol3 = 1'b1;
         localparam ol4 = 16'b0000000000000000;
         begin
            or14 = arg_0;
            or1 = arg_1;
            or6 = arg_2;
            or0 = or1[0:0];
            or2 = or0 == ol0;
            or3 = or1[1:1];
            or4 = or3 == ol1;
            or5 = or2 & or4;
            or7 = or5 ? or6 : ol2;
            or8 = or1[1:1];
            or9 = or8 == ol3;
            or10 = or1[17:2];
            or11 = or9 ? or10 : or6;
            or12 = ol4;
            or12[15:0] = or11;
            or13 = {or12, or7};
            kernel_register_kernel = or13;
         end
   endfunction
endmodule
module top_memory(input wire [1:0] clock_reset, input wire [15:0] i, output reg [15:0] o);
   wire  clock;
   wire  reset;
   assign clock = clock_reset[0];
   assign reset = clock_reset[1];
   initial begin
      o = 16'b0000000000000000;
   end
   always @(posedge clock) begin
      if (reset) begin
         o <= 16'b0000000000000000;
      end else begin
         o <= i;
      end
   end
endmodule
