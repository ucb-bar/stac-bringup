open_hw_manager
connect_hw_server -url localhost:3121
current_hw_target [get_hw_targets */xilinx_tcf/Digilent/210319B7CC01A]
set_property PARAM.FREQUENCY 15000000 [get_hw_targets */xilinx_tcf/Digilent/210319B7CC01A]
open_hw_target
set_property PROGRAM.FILE {Arty100THarness.bit} [lindex [get_hw_devices] 0]
program_hw_devices [lindex [get_hw_devices] 0]

