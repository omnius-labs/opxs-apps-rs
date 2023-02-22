import com.dimafeng.testcontainers.ForAllTestContainer
import com.dimafeng.testcontainers.PostgreSQLContainer
import omnius.pxtv.migration.Migrator
import org.scalatest.funsuite.AnyFunSuite

import java.nio.file.NoSuchFileException
import java.nio.file.Paths

class MigratorTest extends AnyFunSuite with ForAllTestContainer {
  override val container: PostgreSQLContainer = PostgreSQLContainer()
  test("simple_create_table") {
    val migrator =
      new Migrator(
        Paths.get("cases/simple_create_table").toAbsolutePath.toString,
        "test"
      )
    assertThrows[NoSuchFileException] { // Result type: Assertion
      migrator.execute(container.jdbcUrl, container.username, container.password)
    }
  }
  test("create_table_syntax_error") {
    val migrator =
      new Migrator(
        Paths.get("cases/create_table_syntax_error").toAbsolutePath.toString,
        "test"
      )
    assertThrows[NoSuchFileException] { // Result type: Assertion
      migrator.execute(container.jdbcUrl, container.username, container.password)
    }
  }
}
