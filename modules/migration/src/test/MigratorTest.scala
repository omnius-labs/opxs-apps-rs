import com.dimafeng.testcontainers.{ForAllTestContainer, PostgreSQLContainer}
import org.scalatest.funsuite.AnyFunSuite
import omnius.pxtv.migration.{PostgresOptions, Migrator}

class MigratorTest extends AnyFunSuite with ForAllTestContainer {
  override val container: PostgreSQLContainer = PostgreSQLContainer()
  test("normal") {
    val path = System.getProperty("user.dir") + "/../../conf/db/migrations";
    val migrator =
      new Migrator(path, PostgresOptions(container.jdbcUrl, container.username, container.password))
    migrator.execute()
  }
}
