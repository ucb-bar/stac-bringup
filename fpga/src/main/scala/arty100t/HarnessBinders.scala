package chipyard.fpga.arty100t

import chisel3._
import chisel3.experimental.{DataMirror, Direction}

import freechips.rocketchip.jtag.{JTAGIO}
import freechips.rocketchip.subsystem.{PeripheryBusKey}
import freechips.rocketchip.tilelink.{TLBundle}
import freechips.rocketchip.util.{HeterogeneousBag}
import freechips.rocketchip.diplomacy.{LazyRawModuleImp}

import sifive.blocks.devices.uart.{UARTPortIO, HasPeripheryUARTModuleImp, UARTParams}
import sifive.blocks.devices.jtag.{JTAGPins, JTAGPinsFromPort}
import sifive.blocks.devices.pinctrl.{BasePin}
import sifive.fpgashells.shell._
import sifive.fpgashells.ip.xilinx._
import sifive.fpgashells.shell.xilinx._
import sifive.fpgashells.clocks._
import chipyard._
import chipyard.harness._
import chipyard.iobinders._

import testchipip._

class WithArty100TUARTTSI extends HarnessBinder({
  case (th: HasHarnessInstantiators, port: UARTTSIPort) => {
    val ath = th.asInstanceOf[LazyRawModuleImp].wrapper.asInstanceOf[Arty100THarness]
    ath.io_uart_bb.bundle <> port.io.uart
    ath.other_leds(1) := port.io.dropped
    ath.other_leds(9) := port.io.tsi2tl_state(0)
    ath.other_leds(10) := port.io.tsi2tl_state(1)
    ath.other_leds(11) := port.io.tsi2tl_state(2)
    ath.other_leds(12) := port.io.tsi2tl_state(3)
  }
})

class WithArty100TDDRTL extends HarnessBinder({
  case (th: HasHarnessInstantiators, port: TLMemPort) => {
    val artyTh = th.asInstanceOf[LazyRawModuleImp].wrapper.asInstanceOf[Arty100THarness]
    val bundles = artyTh.ddrClient.out.map(_._1)
    val ddrClientBundle = Wire(new HeterogeneousBag(bundles.map(_.cloneType)))
    bundles.zip(ddrClientBundle).foreach { case (bundle, io) => bundle <> io }
    ddrClientBundle <> port.io
  }
})

// Uses PMOD JA/JB
class WithArty100TSerialTLToGPIO extends HarnessBinder({
  case (th: HasHarnessInstantiators, port: SerialTLPort) => {
    val artyTh = th.asInstanceOf[LazyRawModuleImp].wrapper.asInstanceOf[Arty100THarness]
    val harnessIO = IO(chiselTypeOf(port.io)).suggestName("serial_tl")
    harnessIO <> port.io
    val clkIO = IOPin(harnessIO.clock)
    val packagePinsWithPackageIOs = Seq(
      ("G13", clkIO),
      ("B11", IOPin(harnessIO.bits.out.valid)),
      ("A11", IOPin(harnessIO.bits.out.ready)),
      ("D12", IOPin(harnessIO.bits.out.bits, 0)),
      ("B18", IOPin(harnessIO.bits.in.valid)),
      ("A18", IOPin(harnessIO.bits.in.ready)),
      ("K16", IOPin(harnessIO.bits.in.bits, 0)),
    )
    packagePinsWithPackageIOs foreach { case (pin, io) => {
      artyTh.xdc.addPackagePin(io, pin)
      artyTh.xdc.addIOStandard(io, "LVCMOS33")
    }}

    // Don't add IOB to the clock, if its an input
    if (DataMirror.directionOf(port.io.clock) == Direction.Input) {
      packagePinsWithPackageIOs.drop(1) foreach { case (pin, io) => {
        artyTh.xdc.addIOB(io)
      }}
    } else {
      packagePinsWithPackageIOs foreach { case (pin, io) => {
        artyTh.xdc.addIOB(io)
      }}
    }

    artyTh.sdc.addClock("ser_tl_clock", clkIO, 50)
    artyTh.sdc.addGroup(pins = Seq(clkIO))
    artyTh.xdc.clockDedicatedRouteFalse(clkIO)
  }
})

class WithArty100TStacControllerToGPIO extends HarnessBinder({
  case (th: HasHarnessInstantiators, port: StacControllerPort) => {
    val artyTh = th.asInstanceOf[LazyRawModuleImp].wrapper.asInstanceOf[Arty100THarness]
    val harnessIO = IO(chiselTypeOf(port.io)).suggestName("stac_controller")
    harnessIO <> port.io
    val clkIO = IOPin(harnessIO.clk)
    val packagePinsWithPackageIOs = Seq(
      ("D13", clkIO),
      ("E15", IOPin(harnessIO.reset)),
      ("U12", IOPin(harnessIO.sramScanMode)),
      ("V12", IOPin(harnessIO.pllArstb)),
      ("V10", IOPin(harnessIO.pllScanClk)),
      ("V11", IOPin(harnessIO.pllScanEn)),
      ("U14", IOPin(harnessIO.pllScanIn)),
      ("V14", IOPin(harnessIO.pllScanOut)),
      ("T13", IOPin(harnessIO.pllScanRstn)),
      ("U13", IOPin(harnessIO.pllSel)),
      ("D4", IOPin(harnessIO.sramBistStart)),
      ("D3", IOPin(harnessIO.sramEn)),
      ("F4", IOPin(harnessIO.sramBistEn)),
      ("F3", IOPin(harnessIO.sramExtEn)),
      ("E2", IOPin(harnessIO.sramScanIn)),
      ("D2", IOPin(harnessIO.sramScanOut)),
      ("H2", IOPin(harnessIO.sramScanEn)),
      ("G2", IOPin(harnessIO.sramBistDone)),
    )
    packagePinsWithPackageIOs foreach { case (pin, io) => {
      artyTh.xdc.addPackagePin(io, pin)
      artyTh.xdc.addIOStandard(io, "LVCMOS33")
      artyTh.xdc.addIOB(io)
    }}

    artyTh.sdc.addClock("clock_clock_isolated", clkIO, 50)
    artyTh.sdc.addGroup(pins = Seq(clkIO))
    artyTh.xdc.clockDedicatedRouteFalse(clkIO)
  }
})
