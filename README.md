# STAC Bringup

## Infrastructure

This repo contains source code for generating an Arty100T bitstream for bringup purposes, as well as additional utilities for
running tests on STAC. 

### Compiling bitstream on BWRC servers

To get started, set up Chipyard by following the steps below.

Install `conda` (say "yes" and press enter when prompted):

```
mkdir -m 0700 -p /tools/C/$USER
cd /tools/C/$USER
wget -O Miniforge3.sh \
"https://github.com/conda-forge/miniforge/releases/latest/download/Miniforge3-$(uname)-$(uname -m).sh"
bash Miniforge3.sh -p "/tools/C/${USER}/conda"
```

Then, run Chipyard setup from the root directory of this repo:

```
source ~/.bashrc
mamba install -n base conda-lock=1.4
mamba activate base
./build-setup.sh riscv-tools -s 6 -s 7 -s 8 -s 9 -s 10 --force
./scripts/init-vlsi sky130
```

Source your environment:

```
source env.sh
```

Then, go into the `fpga/` folder and compile the bitstream. Make sure `/tools/xilinx/Vivado/current/bin` is on your `PATH`.

```
make bitstream SUB_PROJECT=arty100t CONFIG=BringupArty100TConfig
```

### Programming FPGA

Copy the bitstream from BWRC onto the lab computer in the `stac-bringup/vivado` folder (currently on the `utils-reorg` branch, will be merged in at some point):

```
scp bwrc:/tools/C/rohankumar/stac-bringup/fpga/generated-src/chipyard.fpga.arty100t.Arty100THarness.BringupArty100TConfig/obj/Arty100THarness.bit /home/rohankumar/stac-bringup/vivado
```

With the FPGA plugged in, run the following from the `stac-bringup/vivado` directory:

```
./program-arty.sh
```

If the FPGA is not found, open up the Vivado Lab GUI by running `vivado_lab` and try to find the device.


### Using the FPGA

Need to write `1` to the register that enables the clock in order to turn on the clock. Another register controls the clock division ratio relative to 50 MHz
(i.e. a value of 125 corresponds to a frequency of 50/125/2 = 200 kHz).

After the clock is turned on, the STAC board's blue UART LED should start flashing. You should then be able to write to MMIO registers that control the SRAM control circuitry
(test SRAMs, BIST, TDCs, delay lines, etc.) via UART TSI.

The current `REGMAP_OFFSET` map:

```
SRAM_EXT_EN -> 0x0
SRAM_SCAN_MODE -> 0x8
SRAM_EN -> 0x10
SRAM_BIST_EN -> 0x18
SRAM_BIST_START -> 0x20
PLL_SEL -> 0x28
PLL_SCAN_RSTN -> 0x30
PLL_ARSTB -> 0x38
SRAM_BIST_DONE -> 0x40
CLK_EN -> 0x48
HALF_CLK_DIV_RATIO -> 0x50
```

These offsets are relative to the STAC controller peripheral's base address, `0x90000000`.

To avoid the UART reset issue, run the following to proxy the FPGA's TTY:

```
sudo socat -d -d /dev/ttyUSB1,raw,echo=0 pty,raw,echo=0
```

Then, point `uarttsi` to the pseudo-TTY instead of directly at the FPGA.

To create a pseudo-TTY for testing purposes, run the following:

```
sudo socat -d -d pty,raw,echo=0 pty,raw,echo=0
```

Then, use `utils/read_tty_raw.py` to listen on one of the created pseudo-TTYs by modifying the `fp_out`
variable:

```
fp_out = open("/dev/ttys011", "rb")
```

Then, run the script using python:

```
python3 read_tty_raw.py
```

When you write to the other pseudo-TTY, the raw bytes should show up in the terminal where you ran the 
Python scripts.

### Using STAC

The current `REGMAP_OFFSET` map for STAC's MMIO registers:

```
ADDR -> 0x0
DIN -> 0x8
MASK -> 0x10
WE -> 0x18
SRAM_ID -> 0x20
SRAM_SEL -> 0x28
SAE_CTL -> 0x30
SAE_SEL -> 0x38
DOUT -> 0x40
TDC -> 0x48
DONE -> 0x68
BIST_RAND_SEED -> 0x70
BIST_SIG_SEED -> 0x80
BIST_MAX_ROW_ADDR -> 0x88
BIST_MAX_COL_ADDR -> 0x90
BIST_INNER_DIM -> 0x98
BIST_ELEMENT_SEQUENCE -> 0xa0
BIST_PATTERN_TABLE -> 0x120
BIST_MAX_ELEMENT_IDX -> 0x140
BIST_CYCLE_LIMIT -> 0x148
BIST_STOP_ON_FAILURE -> 0x150
BIST_FAIL -> 0x158
BIST_FAIL_CYCLE -> 0x160
BIST_EXPECTED -> 0x168
BIST_RECEIVED -> 0x170
BIST_SIGNATURE -> 0x178
EX -> 0x180
```

These offsets are relative to the SRAM BIST peripheral's base address, `0x1000`.

The list of test SRAMs:

```
srams: Seq[SramParams] = Seq(
  new SramParams(8, 8, 2048, 32),
  new SramParams(8, 4, 256, 32),
  new SramParams(8, 4, 64, 32),
  new SramParams(24, 4, 64, 24),
  new SramParams(8, 8, 1024, 32),
  new SramParams(32, 8, 1024, 32),
  new SramParams(32, 4, 512, 32),
  new SramParams(8, 4, 512, 32),
)
```

SRAM select:

```
object SramSrc extends ChiselEnum {
  val mmio = Value(0.U(1.W))
  val bist = Value(1.U(1.W))
}
```

SAE select:

```
object SaeSrc extends ChiselEnum {
  val int = Value(0.U(2.W)) // Internally generated control logic
  val clk = Value(1.U(2.W)) // Off-chip SAE clk
  val ext = Value(2.U(2.W)) // Delay-line generated clk
}
```

The STAC elaboration log is included below

```
Interrupt map (1 harts 2 interrupts):
  [1, 1] => uart_0
  [2, 2] => qspi_0

<stdin>:129.28-134.5: Warning (simple_bus_reg): /soc/subsystem_mbus_clock: missing or empty reg/ranges property
<stdin>:135.28-140.5: Warning (simple_bus_reg): /soc/subsystem_pbus_clock: missing or empty reg/ranges property
<stdin>:36.29-40.6: Warning (interrupt_provider): /cpus/cpu@0/interrupt-controller: Missing #address-cells in interrupt provider
<stdin>:103.36-112.5: Warning (interrupt_provider): /soc/interrupt-controller@c000000: Missing #address-cells in interrupt provider
Clock subsystem_pbus_0: using diplomatically specified frequency of 100.0.
Clock subsystem_mbus_0: using diplomatically specified frequency of 100.0.
120000: Clock domain allClocks_sbus divider
120004: Clock domain allClocks_fbus divider
120008: Clock domain allClocks_rocket divider
12000c: Clock domain allClocks_periph divider

Clock Mux sources:
  0: Some(clkext)
  1: Some(pll_clk_out)
  2: Some(pll_clk_div)
Clock Mux sinks:
  0: allClocks_sbus
  1: allClocks_fbus
  2: allClocks_rocket
  3: allClocks_periph

/dts-v1/;

/ {
	#address-cells = <1>;
	#size-cells = <1>;
	compatible = "ucb-bar,chipyard-dev";
	model = "ucb-bar,chipyard";
	L29: aliases {
		serial0 = &L20;
	};
	L10: chosen {
		stdout-path = &L20;
	};
	L28: cpus {
		#address-cells = <1>;
		#size-cells = <0>;
		timebase-frequency = <1000000>;
		L5: cpu@0 {
			clock-frequency = <0>;
			compatible = "sifive,rocket0", "riscv";
			d-cache-block-size = <64>;
			d-cache-sets = <64>;
			d-cache-size = <4096>;
			device_type = "cpu";
			hardware-exec-breakpoint-count = <1>;
			i-cache-block-size = <64>;
			i-cache-sets = <64>;
			i-cache-size = <4096>;
			next-level-cache = <&L17 &L18 &L19>;
			reg = <0x0>;
			riscv,isa = "rv64imacZicsr_Zifencei_Zihpm_Xrocket";
			riscv,pmpgranularity = <4>;
			riscv,pmpregions = <8>;
			status = "okay";
			timebase-frequency = <1000000>;
			L3: interrupt-controller {
				#interrupt-cells = <1>;
				compatible = "riscv,cpu-intc";
				interrupt-controller;
			};
		};
	};
	L30: htif {
		compatible = "ucb,htif0";
	};
	L18: memory@80000000 {
		device_type = "memory";
		reg = <0x80000000 0x40000000>;
	};
	L27: soc {
		#address-cells = <1>;
		#size-cells = <1>;
		compatible = "ucb-bar,chipyard-soc", "simple-bus";
		ranges;
		L21: QSPI@10021000 {
			#address-cells = <1>;
			#size-cells = <0>;
			clocks = <&L0>;
			compatible = "sifive, QSPI0";
			interrupt-parent = <&L6>;
			interrupts = <2>;
			reg = <0x10021000 0x1000 0x20000000 0x10000000>;
			reg-names = "control", "mem";
		};
		L22: SramBist@1000 {
			reg = <0x1000 0x1000>;
			reg-names = "control";
		};
		L17: backing-scratchpad@8000000 {
			compatible = "sifive,sram0";
			reg = <0x8000000 0x1000>;
			reg-names = "mem";
		};
		L16: boot-address-reg@4000 {
			reg = <0x4000 0x1000>;
			reg-names = "control";
		};
		L7: clint@2000000 {
			compatible = "riscv,clint0";
			interrupts-extended = <&L3 3 &L3 7>;
			reg = <0x2000000 0x10000>;
			reg-names = "control";
		};
		L25: clk-div-ctrl@120000 {
			reg = <0x120000 0x1000>;
			reg-names = "control";
		};
		L23: clock-gater@100000 {
			reg = <0x100000 0x1000>;
			reg-names = "control";
		};
		L8: debug-controller@0 {
			compatible = "sifive,debug-013", "riscv,debug-013";
			debug-attach = "jtag";
			interrupts-extended = <&L3 65535>;
			reg = <0x0 0x1000>;
			reg-names = "control";
		};
		L1: error-device@3000 {
			compatible = "sifive,error0";
			reg = <0x3000 0x1000>;
		};
		L6: interrupt-controller@c000000 {
			#interrupt-cells = <1>;
			compatible = "riscv,plic0";
			interrupt-controller;
			interrupts-extended = <&L3 11>;
			reg = <0xc000000 0x4000000>;
			reg-names = "control";
			riscv,max-priority = <3>;
			riscv,ndev = <2>;
		};
		L19: lbwif-rom@20000 {
			reg = <0x20000 0x10000>;
		};
		L15: rom@10000 {
			compatible = "sifive,rom0";
			reg = <0x10000 0x10000>;
			reg-names = "mem";
		};
		L20: serial@10020000 {
			clocks = <&L0>;
			compatible = "sifive,uart0";
			interrupt-parent = <&L6>;
			interrupts = <1>;
			reg = <0x10020000 0x1000>;
			reg-names = "control";
		};
		L2: subsystem_mbus_clock {
			#clock-cells = <0>;
			clock-frequency = <100000000>;
			clock-output-names = "subsystem_mbus_clock";
			compatible = "fixed-clock";
		};
		L0: subsystem_pbus_clock {
			#clock-cells = <0>;
			clock-frequency = <100000000>;
			clock-output-names = "subsystem_pbus_clock";
			compatible = "fixed-clock";
		};
		L24: tile-reset-setter@110000 {
			reg = <0x110000 0x1000>;
			reg-names = "control";
		};
	};
};

Generated Address Map
	      0 -     1000 ARWX  debug-controller@0
	   1000 -     2000  RW   SramBist@1000
	   3000 -     4000 ARWX  error-device@3000
	   4000 -     5000 ARW   boot-address-reg@4000
	  10000 -    20000  R X  rom@10000
	  20000 -    30000  R XC lbwif-rom@20000
	 100000 -   101000 ARW   clock-gater@100000
	 110000 -   111000 ARW   tile-reset-setter@110000
	 120000 -   121000 ARW   clk-div-ctrl@120000
	2000000 -  2010000 ARW   clint@2000000
	8000000 -  8001000  RWXC backing-scratchpad@8000000
	c000000 - 10000000 ARW   interrupt-controller@c000000
	10020000 - 10021000 ARW   serial@10020000
	10021000 - 10022000 ARW   QSPI@10021000
	20000000 - 30000000 ARWX  QSPI@10021000
	80000000 - c0000000  RWXC memory@80000000

IOCells generated by IOBinders:
  IOBinder for interface testchipip.CanHavePeripheryTLSerial generated:
    3 X Sky130EFGPIOV2CellIn
      iocell_serial_tl_bits_out_ready <-> serial_tl_bits_out_ready
      iocell_serial_tl_bits_in_bits <-> serial_tl_bits_in_bits
      iocell_serial_tl_bits_in_valid <-> serial_tl_bits_in_valid
    4 X Sky130EFGPIOV2CellOut
      iocell_serial_tl_bits_out_bits <-> serial_tl_bits_out_bits
      iocell_serial_tl_bits_out_valid <-> serial_tl_bits_out_valid
      iocell_serial_tl_bits_in_ready <-> serial_tl_bits_in_ready
      iocell_serial_tl_clock <-> serial_tl_clock
  IOBinder for interface testchipip.CanHavePeripheryCustomBootPin generated:
    1 X Sky130EFGPIOV2CellIn
      iocell_custom_boot <-> custom_boot
  IOBinder for interface srambist.HasPeripherySramBistModuleImp generated:
    8 X Sky130EFGPIOV2CellIn
      iocell_sram_bist_bistStart <-> sram_bist_bistStart
      iocell_sram_bist_bistEn <-> sram_bist_bistEn
      iocell_sram_bist_sramSaeClk <-> sram_bist_sramSaeClk
      iocell_sram_bist_sramScanEn <-> sram_bist_sramScanEn
      iocell_sram_bist_sramScanIn <-> sram_bist_sramScanIn
      iocell_sram_bist_sramEn <-> sram_bist_sramEn
      iocell_sram_bist_sramScanMode <-> sram_bist_sramScanMode
      iocell_sram_bist_sramExtEn <-> sram_bist_sramExtEn
    2 X Sky130EFGPIOV2CellOut
      iocell_sram_bist_bistDone <-> sram_bist_bistDone
      iocell_sram_bist_sramScanOut <-> sram_bist_sramScanOut
  IOBinder for interface chipyard.clocking.HasChipyardPRCI generated:
    9 X Sky130EFGPIOV2CellIn
      iocell_clock_clock <-> clock_clock
      iocell_clksel_0 <-> clksel_0
      iocell_clksel_1 <-> clksel_1
      iocell_pll_sel <-> pll_sel
      iocell_pll_arstb <-> pll_arstb
      iocell_pll_scan_rst <-> pll_scan_rst
      iocell_pll_scan_en <-> pll_scan_en
      iocell_pll_scan_clk <-> pll_scan_clk
      iocell_pll_scan_in <-> pll_scan_in
    1 X Sky130EFGPIOV2CellOut
      iocell_pll_scan_out <-> pll_scan_out
    3 X Sky130EFAnalogCellIOCell
      iocell_pll_ref_in <-> pll_ref_in
      iocell_pll_clk_out <-> pll_clk_out
      iocell_pll_div_out <-> pll_div_out
    1 X Sky130FDXRes4V2IOCell
      iocell_reset <-> reset
  IOBinder for interface sifive.blocks.devices.spi.HasPeripherySPIFlashModuleImp generated:
    4 X Sky130EFGPIOV2CellIO
      iocell_spi_0_dq_0 <-> spi_0_dq_0
      iocell_spi_0_dq_1 <-> spi_0_dq_1
      iocell_spi_0_dq_2 <-> spi_0_dq_2
      iocell_spi_0_dq_3 <-> spi_0_dq_3
    2 X Sky130EFGPIOV2CellOut
      iocell_spi_0_cs_0 <-> spi_0_cs_0
      iocell_spi_0_sck <-> spi_0_sck
  IOBinder for interface sifive.blocks.devices.uart.HasPeripheryUARTModuleImp generated:
    1 X Sky130EFGPIOV2CellIn
      iocell_uart_0_rxd <-> uart_0_rxd
    1 X Sky130EFGPIOV2CellOut
      iocell_uart_0_txd <-> uart_0_txd
  IOBinder for interface freechips.rocketchip.devices.debug.HasPeripheryDebug generated:
    3 X Sky130EFGPIOV2CellIn
      iocell_jtag_TDI <-> jtag_TDI
      iocell_jtag_TMS <-> jtag_TMS
      iocell_jtag_TCK <-> jtag_TCK
    1 X Sky130EFGPIOV2CellOut
      iocell_jtag_TDO <-> jtag_TDO

  Total generated 44 IOCells:
    25 X Sky130EFGPIOV2CellIn
    4 X Sky130EFGPIOV2CellIO
    11 X Sky130EFGPIOV2CellOut
    3 X Sky130EFAnalogCellIOCell
    1 X Sky130FDXRes4V2IOCell

Sky130EFIO: Generated 1 no-conn IO cells/pads
```

## Log

### 2/15/24

- Ran preliminary tests on the delay lines and TDCs. Was able to see changing TDC code in
  response to configuring delay line.

### 2/13/24

- Began testing test SRAMs and working on UART TSI

TODOs:
- Bring up TDC / delay line to the point where we can run basic sanity checks
- Write documentation on how to test TDC / delay line
  (e.g. increasing the delay line code results in an increasing TDC code).
- Test the smaller test SRAMs
- Run rand test pattern on small test SRAMs
- Write documentation on how to test UART TSI if needed

### 2/12/24

Ran more thorough PEX simulations on `sram22_512x64m4w8`.
Found that in the `ff` corner, the `wl_en0` (unbuffered wordline enable)
pulse is too short, causing read/write issues.
The pulse length is controlled by an inverter chain. In the `ff` corner,
the inverter chain is fast, and the pulse length becomes too short.
We need better matching between the delay to drive the replica bitcells
and the inverter chain that controls wordline pulse length.

No issues were identified in the `tt` corner.
The bitline swing in PEX simulation in the `tt` corner is approximately 0.55V.
Sense amp enable timing could be a bit faster.

### 2/8/24

- Ran `bebe_host.py` SRAM BIST with random pattern on STAC board 2
  ```
  WRITE 0x0 to all addresses
  RANDOM READ/WRITE
  ```
- Failures in several addresses, unclear if they are correlated

### 2/6/24

- Ran `bebe_host.py` SRAM BIST with MATS+ pattern on two STAC boards
  ```
  WRITE 0x0
  READ 0x0 + WRITE 0xffffffffffffffff
  READ 0xffffffffffffffff + WRITE 0x0
  ```
- Boards 1 and 2 had identical failures:
  ```
  ERROR reading 0x1ef: got 0x5200000000000000, expected 0x0
  ERROR reading 0x1f0: got 0x8000000000000000, expected 0x0
  ERROR reading 0x1f1: got 0x9002010000000000, expected 0x0
  ERROR reading 0x1f1: got 0x9002010000000000, expected 0xffffffffffffffff
  ERROR reading 0x1f0: got 0x8000000000000000, expected 0xffffffffffffffff
  ERROR reading 0x1ef: got 0x5200000000000000, expected 0xffffffffffffffff
  ```
- Hypotheses:
  - Distance to nearest row/n-well tap
  - Too far away from write circuitry
  - Decoder one-hot is incorrect

![CHIPYARD](https://github.com/ucb-bar/chipyard/raw/main/docs/_static/images/chipyard-logo-full.png)

# Chipyard Framework [![Test](https://github.com/ucb-bar/chipyard/actions/workflows/chipyard-run-tests.yml/badge.svg)](https://github.com/ucb-bar/chipyard/actions)

## Quick Links

* **Stable Documentation**: https://chipyard.readthedocs.io/
* **User Question Forum**: https://groups.google.com/forum/#!forum/chipyard
* **Bugs and Feature Requests**: https://github.com/ucb-bar/chipyard/issues

## Using Chipyard

To get started using Chipyard, see the stable documentation on the Chipyard documentation site: https://chipyard.readthedocs.io/

## What is Chipyard

Chipyard is an open source framework for agile development of Chisel-based systems-on-chip.
It will allow you to leverage the Chisel HDL, Rocket Chip SoC generator, and other [Berkeley][berkeley] projects to produce a [RISC-V][riscv] SoC with everything from MMIO-mapped peripherals to custom accelerators.
Chipyard contains processor cores ([Rocket][rocket-chip], [BOOM][boom], [CVA6 (Ariane)][cva6]), accelerators ([Hwacha][hwacha], [Gemmini][gemmini], [NVDLA][nvdla]), memory systems, and additional peripherals and tooling to help create a full featured SoC.
Chipyard supports multiple concurrent flows of agile hardware development, including software RTL simulation, FPGA-accelerated simulation ([FireSim][firesim]), automated VLSI flows ([Hammer][hammer]), and software workload generation for bare-metal and Linux-based systems ([FireMarshal][firemarshal]).
Chipyard is actively developed in the [Berkeley Architecture Research Group][ucb-bar] in the [Electrical Engineering and Computer Sciences Department][eecs] at the [University of California, Berkeley][berkeley].

## Resources

* Chipyard Stable Documentation: https://chipyard.readthedocs.io/
* Chipyard (x FireSim) Tutorial: https://fires.im/tutorial-recent/
* Chipyard Basics slides: https://fires.im/asplos23-slides-pdf/02_chipyard_basics.pdf

## Need help?

* Join the Chipyard Mailing List: https://groups.google.com/forum/#!forum/chipyard
* If you find a bug or would like propose a feature, post an issue on this repo: https://github.com/ucb-bar/chipyard/issues

## Contributing

* See [CONTRIBUTING.md](/CONTRIBUTING.md)

## Attribution and Chipyard-related Publications

If used for research, please cite Chipyard by the following publication:

```
@article{chipyard,
  author={Amid, Alon and Biancolin, David and Gonzalez, Abraham and Grubb, Daniel and Karandikar, Sagar and Liew, Harrison and Magyar,   Albert and Mao, Howard and Ou, Albert and Pemberton, Nathan and Rigge, Paul and Schmidt, Colin and Wright, John and Zhao, Jerry and Shao, Yakun Sophia and Asanovi\'{c}, Krste and Nikoli\'{c}, Borivoje},
  journal={IEEE Micro},
  title={Chipyard: Integrated Design, Simulation, and Implementation Framework for Custom SoCs},
  year={2020},
  volume={40},
  number={4},
  pages={10-21},
  doi={10.1109/MM.2020.2996616},
  ISSN={1937-4143},
}
```

* **Chipyard**
    * A. Amid, et al. *IEEE Micro'20* [PDF](https://ieeexplore.ieee.org/document/9099108).
    * A. Amid, et al. *DAC'20* [PDF](https://ieeexplore.ieee.org/document/9218756).
    * A. Amid, et al. *ISCAS'21* [PDF](https://ieeexplore.ieee.org/abstract/document/9401515).

These additional publications cover many of the internal components used in Chipyard. However, for the most up-to-date details, users should refer to the Chipyard docs.

* **Generators**
    * **Rocket Chip**: K. Asanovic, et al., *UCB EECS TR*. [PDF](http://www2.eecs.berkeley.edu/Pubs/TechRpts/2016/EECS-2016-17.pdf).
    * **BOOM**: C. Celio, et al., *Hot Chips 30*. [PDF](https://old.hotchips.org/hc30/1conf/1.03_Berkeley_BROOM_HC30.Berkeley.Celio.v02.pdf).
      * **SonicBOOM (BOOMv3)**: J. Zhao, et al., *CARRV'20*. [PDF](https://carrv.github.io/2020/papers/CARRV2020_paper_15_Zhao.pdf).
      * **COBRA (BOOM Branch Prediction)**: J. Zhao, et al., *ISPASS'21*. [PDF](https://ieeexplore.ieee.org/document/9408173).
    * **Hwacha**: Y. Lee, et al., *ESSCIRC'14*. [PDF](http://hwacha.org/papers/riscv-esscirc2014.pdf).
    * **Gemmini**: H. Genc, et al., *DAC'21*. [PDF](https://arxiv.org/pdf/1911.09925).
* **Sims**
    * **FireSim**: S. Karandikar, et al., *ISCA'18*. [PDF](https://sagark.org/assets/pubs/firesim-isca2018.pdf).
        * **FireSim Micro Top Picks**: S. Karandikar, et al., *IEEE Micro, Top Picks 2018*. [PDF](https://sagark.org/assets/pubs/firesim-micro-top-picks2018.pdf).
        * **FASED**: D. Biancolin, et al., *FPGA'19*. [PDF](https://people.eecs.berkeley.edu/~biancolin/papers/fased-fpga19.pdf).
        * **Golden Gate**: A. Magyar, et al., *ICCAD'19*. [PDF](https://davidbiancolin.github.io/papers/goldengate-iccad19.pdf).
        * **FirePerf**: S. Karandikar, et al., *ASPLOS'20*. [PDF](https://sagark.org/assets/pubs/fireperf-asplos2020.pdf).
        * **FireSim ISCA@50 Retrospective**: S. Karandikar, et al., *ISCA@50 Retrospective: 1996-2020*. [PDF](https://sites.coecis.cornell.edu/isca50retrospective/files/2023/06/Karandikar_2018_FireSim.pdf)
* **Tools**
    * **Chisel**: J. Bachrach, et al., *DAC'12*. [PDF](https://people.eecs.berkeley.edu/~krste/papers/chisel-dac2012.pdf).
    * **FIRRTL**: A. Izraelevitz, et al., *ICCAD'17*. [PDF](https://ieeexplore.ieee.org/document/8203780).
    * **Chisel DSP**: A. Wang, et al., *DAC'18*. [PDF](https://ieeexplore.ieee.org/document/8465790).
    * **FireMarshal**: N. Pemberton, et al., *ISPASS'21*. [PDF](https://ieeexplore.ieee.org/document/9408192).
* **VLSI**
    * **Hammer**: E. Wang, et al., *ISQED'20*. [PDF](https://www.isqed.org/English/Archives/2020/Technical_Sessions/113.html).
    * **Hammer**: H. Liew, et al., *DAC'22*. [PDF](https://dl.acm.org/doi/abs/10.1145/3489517.3530672).

## Acknowledgements

This work is supported by the NSF CCRI ENS Chipyard Award #2016662.

[hwacha]:https://www2.eecs.berkeley.edu/Pubs/TechRpts/2015/EECS-2015-262.pdf
[hammer]:https://github.com/ucb-bar/hammer
[firesim]:https://fires.im
[ucb-bar]: http://bar.eecs.berkeley.edu
[eecs]: https://eecs.berkeley.edu
[berkeley]: https://berkeley.edu
[riscv]: https://riscv.org/
[rocket-chip]: https://github.com/freechipsproject/rocket-chip
[boom]: https://github.com/riscv-boom/riscv-boom
[firemarshal]: https://github.com/firesim/FireMarshal/
[cva6]: https://github.com/openhwgroup/cva6/
[gemmini]: https://github.com/ucb-bar/gemmini
[nvdla]: http://nvdla.org/
