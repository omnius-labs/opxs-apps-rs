import com.dimafeng.testcontainers.ForAllTestContainer
import com.dimafeng.testcontainers.PostgreSQLContainer
import omnius.pxtv.migration.Migrator
import omnius.pxtv.migration.PostgresOptions
import org.scalatest.funsuite.AnyFunSuite

import java.nio.file.Paths

class MigratorTest extends AnyFunSuite with ForAllTestContainer {
  override val container: PostgreSQLContainer = PostgreSQLContainer()
  test("normal") {
    val path = Paths.get("../../migrations").toAbsolutePath.toString;
    val migrator =
      new Migrator(path, PostgresOptions(container.jdbcUrl, container.username, container.password))
    migrator.execute()
  }
}
