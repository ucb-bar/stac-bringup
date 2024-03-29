package staccontroller

import chisel3._
import chisel3.util._
import org.chipsalliance.cde.config.Parameters
import freechips.rocketchip.diplomacy._
import freechips.rocketchip.interrupts._
import freechips.rocketchip.prci._
import freechips.rocketchip.regmapper._
import freechips.rocketchip.subsystem._
import freechips.rocketchip.tilelink._
import freechips.rocketchip.devices.tilelink._
import freechips.rocketchip.util._

import staccontroller.StacControllerCtrlRegs._

class StacControllerTopIO extends Bundle {
  val sramExtEn = Output(Bool())
  val sramScanMode = Output(Bool())
  val sramEn = Output(Bool())
  val sramScanIn = Output(Bool())
  val sramScanEn = Output(Bool())
  val sramBistEn = Output(Bool())
  val sramBistStart = Output(Bool())
  val pllSel = Output(Bool())
  val pllScanEn = Output(Bool())
  val pllScanRstn = Output(Bool())
  val pllScanClk = Output(Bool())
  val pllScanIn = Output(Bool())
  val pllArstb = Output(Bool())
  val sramScanOut = Input(Bool())
  val sramBistDone = Input(Bool())
  val pllScanOut = Input(Bool())
  val reset = Output(Bool())
  val clk = Output(Bool())
}

class StacControllerIO extends Bundle {
  val top = new StacControllerTopIO
  val mmio = new StacControllerMmioRegIO
}

class StacController(params: StacControllerParams)(implicit p: Parameters) extends Module {
  val io = IO(new StacControllerIO)

  val sramExtEn = RegInit(false.B)
  val sramScanMode = RegInit(false.B)
  val sramEn = RegInit(false.B)
  val sramBistEn = RegInit(false.B)
  val sramBistStart = RegInit(false.B)
  val pllSel = RegInit(false.B)
  val pllScanRstn = RegInit(false.B)
  val pllArstb = RegInit(false.B)
  val halfClkDivRatio = RegInit(params.halfClkDivRatio.U(32.W))
  val clkEn = RegInit(false.B)
  val divClk = RegInit(false.B)
  val cycles = RegInit(0.U(32.W))

  Seq(
    (sramExtEn, io.top.sramExtEn, io.mmio.sramExtEn),
    (sramScanMode, io.top.sramScanMode, io.mmio.sramScanMode),
    (sramEn, io.top.sramEn, io.mmio.sramEn),
    (sramBistEn, io.top.sramBistEn, io.mmio.sramBistEn),
    (sramBistStart, io.top.sramBistStart, io.mmio.sramBistStart),
    (pllSel, io.top.pllSel, io.mmio.pllSel),
    (pllScanRstn, io.top.pllScanRstn, io.mmio.pllScanRstn),
    (pllArstb, io.top.pllArstb, io.mmio.pllArstb),
    ).foreach { case (reg, io_top, io_mmio) => {
        when(io_mmio.en) {
          reg := io_mmio.d
        }
        io_mmio.q := reg
        io_top := reg
      }
    }

    Seq(
      (clkEn, io.mmio.clkEn),
      (halfClkDivRatio, io.mmio.halfClkDivRatio),
      ).foreach { case (reg, io_mmio) => {
        when (io_mmio.en) {
          reg := io_mmio.d
        }
        io_mmio.q := reg
      }
    }


  io.top.sramScanIn := true.B 
  io.top.sramScanEn := false.B 
  io.top.pllScanEn := false.B 
  io.top.pllScanClk := false.B 
  io.top.pllScanIn := true.B 
  io.top.reset := reset

  when (clkEn) {
    io.top.clk := divClk
  } .otherwise {
    io.top.clk := false.B
  }

  when (halfClkDivRatio === 0.U || cycles >= halfClkDivRatio - 1.U) {
    cycles := 0.U
    divClk := ~divClk
  } .otherwise {
    cycles := cycles + 1.U
  }

  io.mmio.sramBistDone.q := io.top.sramBistDone
}

abstract class StacControllerRouter(busWidthBytes: Int, params: StacControllerParams)(
    implicit p: Parameters
) extends IORegisterRouter(
      RegisterRouterParams(
        name = "StacController",
        compat = Seq(),
        base = params.address,
        beatBytes = busWidthBytes
      ),
      new StacControllerTopIO
    ) {

  lazy val module = new LazyModuleImp(this) {
    val io = ioNode.bundle

    val stacController = Module(new StacController(params))

    io <> stacController.io.top

    regmap(
      REGMAP_OFFSET(SRAM_EXT_EN) -> Seq(
        RegField.rwReg(REG_WIDTH(SRAM_EXT_EN), stacController.io.mmio.sramExtEn)
      ),
      REGMAP_OFFSET(SRAM_SCAN_MODE) -> Seq(
        RegField.rwReg(REG_WIDTH(SRAM_SCAN_MODE), stacController.io.mmio.sramScanMode)
      ),
      REGMAP_OFFSET(SRAM_EN) -> Seq(
        RegField.rwReg(REG_WIDTH(SRAM_EN), stacController.io.mmio.sramEn)
      ),
      REGMAP_OFFSET(SRAM_BIST_EN) -> Seq(
        RegField.rwReg(REG_WIDTH(SRAM_BIST_EN), stacController.io.mmio.sramBistEn)
      ),
      REGMAP_OFFSET(SRAM_BIST_START) -> Seq(
        RegField.rwReg(REG_WIDTH(SRAM_BIST_START), stacController.io.mmio.sramBistStart)
      ),
      REGMAP_OFFSET(PLL_SEL) -> Seq(
        RegField.rwReg(REG_WIDTH(PLL_SEL), stacController.io.mmio.pllSel)
      ),
      REGMAP_OFFSET(PLL_SCAN_RSTN) -> Seq(
        RegField.rwReg(REG_WIDTH(PLL_SCAN_RSTN), stacController.io.mmio.pllScanRstn)
      ),
      REGMAP_OFFSET(PLL_ARSTB) -> Seq(
        RegField.rwReg(REG_WIDTH(PLL_ARSTB), stacController.io.mmio.pllArstb)
      ),
      REGMAP_OFFSET(SRAM_BIST_DONE) -> Seq(
        RegField.rwReg(REG_WIDTH(SRAM_BIST_DONE), stacController.io.mmio.sramBistDone)
      ),
      REGMAP_OFFSET(CLK_EN) -> Seq(
        RegField.rwReg(REG_WIDTH(CLK_EN), stacController.io.mmio.clkEn)
      ),
      REGMAP_OFFSET(HALF_CLK_DIV_RATIO) -> Seq(
        RegField.rwReg(REG_WIDTH(HALF_CLK_DIV_RATIO), stacController.io.mmio.halfClkDivRatio)
      ),
    )
  }
}

class TLStacController(busWidthBytes: Int, params: StacControllerParams)(implicit
    p: Parameters
) extends StacControllerRouter(busWidthBytes, params)
    with HasTLControlRegMap

case class StacControllerAttachParams(
    device: StacControllerParams,
    controlWhere: TLBusWrapperLocation = SBUS,
    blockerAddr: Option[BigInt] = None,
    controlXType: ClockCrossingType = NoCrossing,
) {
  def attachTo(where: Attachable)(implicit p: Parameters): TLStacController = {
    val name = s"stac_controller_${StacController.nextId()}"
    val tlbus = where.locateTLBusWrapper(controlWhere)
    val stacControllerClockDomainWrapper = LazyModule(
      new ClockSinkDomain(take = None)
    )
    val stacController = stacControllerClockDomainWrapper {
      LazyModule(new TLStacController(tlbus.beatBytes, device))
    }
    stacController.suggestName(name)

    tlbus.coupleTo(s"device_named_$name") { bus =>

      val blockerOpt = blockerAddr.map { a =>
        val blocker = LazyModule(
          new TLClockBlocker(
            BasicBusBlockerParams(a, tlbus.beatBytes, tlbus.beatBytes)
          )
        )
        tlbus.coupleTo(s"bus_blocker_for_$name") {
          blocker.controlNode := TLFragmenter(tlbus) := _
        }
        blocker
      }

      stacControllerClockDomainWrapper.clockNode := (controlXType match {
        case _: SynchronousCrossing =>
          tlbus.dtsClk.map(_.bind(stacController.device))
          tlbus.fixedClockNode
        case _: RationalCrossing =>
          tlbus.clockNode
        case _: AsynchronousCrossing =>
          val stacControllerClockGroup = ClockGroup()
          stacControllerClockGroup := where.asyncClockGroupsNode
          blockerOpt.map { _.clockNode := stacControllerClockGroup }.getOrElse {
            stacControllerClockGroup
          }
      })

      (stacController.controlXing(controlXType)
        := TLFragmenter(tlbus)
        := blockerOpt.map { _.node := bus }.getOrElse { bus })
    }

    stacController
  }
}

object StacController {
  val nextId = { var i = -1; () => { i += 1; i } }
}
