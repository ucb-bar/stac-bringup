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

Add the following to the generated `env.sh` file:

```
export PATH=/tools/C/rohankumar/circt/build/bin
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
val res0: Map[StacControllerCtrlRegs.Value, Int] = HashMap(
  SRAM_EXT_EN -> 0, 
  SRAM_SCAN_MODE -> 8, 
  SRAM_EN -> 16, 
  SRAM_BIST_EN -> 24, 
  SRAM_BIST_START -> 32,
  PLL_SEL -> 40, 
  PLL_SCAN_RSTN -> 48, 
  PLL_ARSTB -> 56, 
  SRAM_BIST_DONE -> 64, 
  CLK_EN -> 72, 
  HALF_CLK_DIV_RATIO -> 80, 
)
```

To avoid the UART reset issue, run the following to proxy the FPGA's TTY:

```
sudo socat -d -d /dev/ttyUSB1,raw,echo=0 pty,raw,echo=0
```

Then, point `uarttsi` to the pseudo-TTY instead of directly at the FPGA.

## Log

### 2/13/24

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
