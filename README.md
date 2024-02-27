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
scp bwrc:/tools/C/rohankumar/stac-bringup/fpga/generated-src/chipyard.fpga.arty100t.Arty100THarness.BringupArty100TConfig/obj/Arty100THarness.bit /home/rohan/stac-bringup/vivado
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
  ```
  Reading 0x1ef...        ERROR: got 0x5200000000000000, expected 0x0
  Reading 0x1f0...        ERROR: got 0x800000000000000, expected 0x0
  Reading 0x1f1...        ERROR: got 0x9002010000000000, expected 0x0
  Reading 0x1f0...        ERROR: got 0x800000000000000, expected 0x0
  Reading 0x1b9...        ERROR: got 0x2e13b3766ae17e9c, expected 0x9c7ee16a76b3132e
  Reading 0xed... ERROR: got 0xca1ca1956c3d6f5f, expected 0x5f6f3d6c95a11cca
  Reading 0x1f8...        ERROR: got 0x8abc8b9a635218fc, expected 0xfc1852639a8bbc8a
  Reading 0x159...        ERROR: got 0xc3c2194e18dafd5e, expected 0x5efdda184e19c2c3
  Reading 0x81... ERROR: got 0xda6077d766bf28de, expected 0xde28bf66d77760da
  Reading 0x1f8...        ERROR: got 0x8abc8b9a635218fc, expected 0xfc1852639a8bbc8a
  Reading 0xed... ERROR: got 0xca1ca1956c3d6f5f, expected 0x5f6f3d6c95a11cca
  Reading 0x1d4...        ERROR: got 0x794e45c8640e9a05, expected 0x59a0e64c8454e79
  Reading 0x96... ERROR: got 0x146438681b325f8e, expected 0x8e5f321b68386414
  Reading 0x124...        ERROR: got 0x10c214eadf213b38, expected 0x383b21dfea14c210
  Reading 0x1f8...        ERROR: got 0x8abc8b9a635218fc, expected 0xfc1852639a8bbc8a
  Reading 0x77... ERROR: got 0x5fbc3c68237a9e18, expected 0x189e7a23683cbc5f
  Reading 0x17... ERROR: got 0x8ca591f9ba29f24d, expected 0x4df229baf991a58c
  Reading 0x19e...        ERROR: got 0xb0f9b2b72e592b2a, expected 0x2a2b592eb7b2f9b0
  Reading 0x60... ERROR: got 0xabeb9aec3e99944f, expected 0x4f94993eec9aebab
  Reading 0x1a9...        ERROR: got 0xf72faa10d2e3b6b0, expected 0xb0b6e3d210aa2ff7
  Reading 0x68... ERROR: got 0x11aa8741529e238, expected 0x38e2291574a81a01
  Reading 0xa...  ERROR: got 0xe730537c7b905963, expected 0x6359907b7c5330e7
  Reading 0x199...        ERROR: got 0xceddb5224f6070f7, expected 0xf770604f22b5ddce
  Reading 0x93... ERROR: got 0xd24e9a903c5b81c, expected 0x1cb8c503a9e9240d
  Reading 0x18... ERROR: got 0x894c0248e509e843, expected 0x43e809e548024c89
  Reading 0x157...        ERROR: got 0xf79d627d1e0190a6, expected 0xa690011e7d629df7
  Reading 0x92... ERROR: got 0xc9270bf5b22cac0f, expected 0xfac2cb2f50b27c9
  Reading 0x78... ERROR: got 0xb9a31c37df1e2112, expected 0x12211edf371ca3b9
  Reading 0x44... ERROR: got 0x5ca4d445e1ab4b58, expected 0x584babe145d4a45c
  Reading 0x9d... ERROR: got 0x4e026034ad026f65, expected 0x656f02ad3460024e
  Reading 0x16... ERROR: got 0x29cb31b1bca6ac96, expected 0x96aca6bcb131cb29
  Reading 0x141...        ERROR: got 0xa3d130ef3878dfc8, expected 0xc8df7838ef30d1a3
  Reading 0xaa... ERROR: got 0x838fae6d35b09080, expected 0x8090b0356dae8f83
  Reading 0x1e3...        ERROR: got 0x10dbf84e2d0ffa3f, expected 0x3ffa0f2d4ef8db10
  Reading 0x1b5...        ERROR: got 0x708ae1bc0117ce58, expected 0x58ce1701bce18a70
  Reading 0x1f1...        ERROR: got 0x9002010000000000, expected 0x0
  Reading 0x1de...        ERROR: got 0xccca26ccfb4e2326, expected 0x26234efbcc26cacc
  Reading 0xa4... ERROR: got 0x8228c74b7b46bb3f, expected 0x3fbb467b4bc72882
  Reading 0xa5... ERROR: got 0xb5e636ba168a9e98, expected 0x989e8a16ba36e6b5
  Reading 0x8f... ERROR: got 0x16dc6e8b771d0d6e, expected 0x6e0d1d778b6edc16
  Reading 0xfd... ERROR: got 0x255773b9d7bc5ee0, expected 0xe05ebcd7b9735725
  Reading 0x30... ERROR: got 0x6915943e575fd124, expected 0x24d15f573e941569
  Reading 0x165...        ERROR: got 0x15e51a9bc2b382f4, expected 0xf482b3c29b1ae515
  Reading 0x190...        ERROR: got 0x158967b7f94d6561, expected 0x61654df9b7678915
  Reading 0x1e9...        ERROR: got 0x214aaf0bfa378c0, expected 0xc078a3bff0aa1402
  Reading 0x9d... ERROR: got 0xbcd3d1e79b13d67b, expected 0x7bd6139be7d1d3bc
  Reading 0x61... ERROR: got 0x725f8b1009089f4e, expected 0x4e9f0809108b5f72
  Reading 0x37... ERROR: got 0xbed88912d037cada, expected 0xdaca37d01289d8be
  Reading 0xc2... ERROR: got 0x23016b8b74f90bee, expected 0xee0bf9748b6b0123
  Reading 0xe...  ERROR: got 0x600c5aa0116b9e05, expected 0x59e6b11a05a0c60
  Reading 0x1d2...        ERROR: got 0xfd1e1209f138c107, expected 0x7c138f109121efd
  Reading 0x172...        ERROR: got 0xb99a650c25fb212b, expected 0x2b21fb250c659ab9
  Reading 0x147...        ERROR: got 0x4e8853f665ab1d91, expected 0x911dab65f653884e
  Reading 0x134...        ERROR: got 0x409bb899de5d6ff3, expected 0xf36f5dde99b89b40
  Reading 0x85... ERROR: got 0xee4fb4008abf694a, expected 0x4a69bf8a00b44fee
  Reading 0x1d2...        ERROR: got 0xfd1e1209f138c107, expected 0x7c138f109121efd
  Reading 0x1c8...        ERROR: got 0xcfdca1b3bdf4eff3, expected 0xf3eff4bdb3a1dccf
  Reading 0x1c8...        ERROR: got 0xcfdca1b3bdf4eff3, expected 0xf3eff4bdb3a1dccf
  Reading 0xe2... ERROR: got 0xed67203cdd43b946, expected 0x46b943dd3c2067ed
  Reading 0x151...        ERROR: got 0xb511e7a9764f0229, expected 0x29024f76a9e711b5
  Reading 0x11... ERROR: got 0x1889ec220ba892f3, expected 0xf392a80b22ec8918
  Reading 0xe9... ERROR: got 0xa22a458a101197ae, expected 0xae9711108a452aa2
  Reading 0x10d...        ERROR: got 0x6156a7be12dc530, expected 0x30c52de17b6a1506
  Reading 0x184...        ERROR: got 0x807482b0e741fa50, expected 0x50fa41e7b0827480
  Reading 0x171...        ERROR: got 0xa284b4f66272a7a8, expected 0xa8a77262f6b484a2
  Reading 0xa0... ERROR: got 0x56d0deaeda4b9de2, expected 0xe29d4bdaaeded056
  Reading 0x107...        ERROR: got 0x7aa7c76933b9b43a, expected 0x3ab4b93369c7a77a
  Reading 0x4f... ERROR: got 0xeb77e909145273ce, expected 0xce73521409e977eb
  Reading 0x83... ERROR: got 0x2a4db9695f080c96, expected 0x960c085f69b94d2a
  Reading 0x7c... ERROR: got 0x34a4ada229568140, expected 0x40815629a2ada434
  ```

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
