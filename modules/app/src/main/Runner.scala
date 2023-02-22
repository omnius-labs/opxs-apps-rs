package omnius.pxtv.app

import omnius.pxtv.migration.Migrator
import pureconfig._
import pureconfig.generic.auto._

import java.nio.file.Paths

object Runner {
  def main(args: Array[String]): Unit = {
    val appConfig = ConfigSource.resources("application.conf").loadOrThrow[AppConfig]

    val migrator =
      new Migrator(
        Paths.get("../../migrations").toAbsolutePath.toString,
        "pxtv-app"
      )
    migrator.execute(appConfig.database.jdbcUrl, appConfig.database.username, appConfig.database.password)
  }
}
