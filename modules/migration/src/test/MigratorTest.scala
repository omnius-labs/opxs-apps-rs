package omnius.pxtv.migration

import com.dimafeng.testcontainers.{ForAllTestContainer, PostgreSQLContainer}
import omnius.pxtv.migration.{Migrator, PostgresOptions}
import org.scalatest.funsuite.AnyFunSuite
import java.nio.file.Paths
import java.sql.DriverManager

class MigratorTest extends AnyFunSuite with ForAllTestContainer {
  override val container: PostgreSQLContainer = PostgreSQLContainer()
  test("normal") {
    val migrator = Migrator("./migrations", PostgresOptions(container.jdbcUrl, container.username, container.password))
    migrator.execute()
  }
}
