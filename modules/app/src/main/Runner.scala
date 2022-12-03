package omnius.pxtv.app

import pureconfig.ConfigSource

object Runner {
  def main(args: Array[String]): Unit = {
    val appConfig = ConfigSource.default.load[AppConfig]
  }
}
