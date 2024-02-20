import scala.collection.mutable.LinkedHashMap

object StacControllerCtrlRegs extends Enumeration {
  type Type = Value
  val SRAM_EXT_EN, SRAM_SCAN_MODE, SRAM_EN, SRAM_BIST_EN, 
      SRAM_BIST_START, SRAM_BIST_DONE, PLL_SEL, PLL_SCAN_RSTN, PLL_ARSTB, 
      CLK_EN, HALF_CLK_DIV_RATIO = Value

  val REG_WIDTH = LinkedHashMap(
    SRAM_EXT_EN -> 1,
    SRAM_SCAN_MODE -> 1,
    SRAM_EN -> 1,
    SRAM_BIST_EN -> 1,
    SRAM_BIST_START -> 1,
    PLL_SEL -> 1,
    PLL_SCAN_RSTN -> 1,
    PLL_ARSTB -> 1,
    SRAM_BIST_DONE -> 1,
    CLK_EN -> 1,
    HALF_CLK_DIV_RATIO -> 32,
  )
  val TOTAL_REG_WIDTH = REG_WIDTH.values.sum

  val REGMAP_OFFSET =
    (REG_WIDTH.keys)
      .zip(
        REG_WIDTH.values.scanLeft(0)((acc, n) => acc + ((n - 1) / 64 + 1) * 8)
      )
      .toMap
}

val sorted_regmap = scala.collection.immutable.ListMap(StacControllerCtrlRegs.REGMAP_OFFSET.toSeq.sortBy(_._2):_*).map{case(k,v)=> (k, Integer.toHexString(v))}

