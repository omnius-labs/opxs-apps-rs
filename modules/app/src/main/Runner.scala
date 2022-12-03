package omnius.pxtv.app

import pureconfig._
import pureconfig.generic.auto._

object Runner {
  def main(args: Array[String]): Unit = {
    ConfigSource.default.load[AppConfig]
    val appConfig = ConfigSource.resources("application.conf").loadOrThrow[AppConfig]
  }
}
