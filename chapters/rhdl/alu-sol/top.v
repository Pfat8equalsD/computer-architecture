module top(input wire [1:0] clock_reset, input wire [0:0] i, output wire [1:0] o);
   wire [3:0] od;
   wire [1:0] d;
   wire [1:0] q;
   assign o = od[1:0];
   top_state c0(.clock_reset(clock_reset), .i(d[1:0]), .o(q[1:0]));
   assign d = od[3:2];
   assign od = kernel_kernel(clock_reset, i, q);
   function [3:0] kernel_kernel(input reg [1:0] arg_0, input reg [0:0] arg_1, input reg [1:0] arg_2);
         reg [1:0] or0;
         reg [2:0] or1;
         reg [0:0] or2;
         reg [2:0] or3;
         reg [0:0] or4;
         reg [1:0] or5;
         reg [1:0] or6;
         reg [1:0] or7;
         reg [1:0] or8;
         reg [3:0] or9;
         reg [1:0] or10;
         localparam ol0 = 3'b000;
         localparam ol1 = 3'b010;
         localparam ol2 = 3'b100;
         localparam ol3 = 3'b000;
         localparam ol4 = 3'b001;
         localparam ol5 = 3'b100;
         localparam ol6 = 3'b101;
         localparam ol7 = 3'b000;
         localparam ol8 = 3'b010;
         localparam ol9 = 3'b100;
         localparam ol10 = 3'b110;
         localparam ol11 = 3'b001;
         localparam ol12 = 2'b00;
         localparam ol13 = 2'b00;
         begin
            or10 = arg_0;
            or2 = arg_1;
            or0 = arg_2;
            or1 = {or2, or0};
            case (or1)
               3'b000 : or3 = ol1;
               3'b100 : or3 = ol3;
               3'b001 : or3 = ol5;
               3'b101 : or3 = ol7;
               3'b010 : or3 = ol9;
               3'b110 : or3 = ol11;
            endcase
            or4 = or3[0:0];
            or5 = or3[2:1];
            or6 = ol12;
            or6[0:0] = or4;
            or7 = or6;
            or7[1:1] = or4;
            or8 = ol13;
            or8[1:0] = or5;
            or9 = {or8, or7};
            kernel_kernel = or9;
         end
   endfunction
endmodule
module top_state(input wire [1:0] clock_reset, input wire [1:0] i, output reg [1:0] o);
   wire  clock;
   wire  reset;
   assign clock = clock_reset[0];
   assign reset = clock_reset[1];
   initial begin
      o = 2'b00;
   end
   always @(posedge clock) begin
      if (reset) begin
         o <= 2'b00;
      end else begin
         o <= i;
      end
   end
endmodule
