package omnius.pxtv.migration

import java.nio.file.Files
import java.nio.file.Paths
import java.sql.Connection
import java.sql.DriverManager
import java.sql.SQLException
import java.time.LocalDateTime
import scala.collection.immutable.HashSet
import scala.collection.immutable.List
import scala.collection.mutable.ListBuffer
import scala.jdk.CollectionConverters._

class Migrator(
    val basedir: String,
    val postgres: PostgresOptions
) {
  def execute(): Unit = {
    val con = DriverManager.getConnection(postgres.jdbcUrl, postgres.username, postgres.password)

    try {
      this.createTables(con)

      val files = this.loadMigrationFiles(this.basedir)
      val histories = this.fetchMigrationHistories(con)

      val ignoreSet = HashSet.from(histories.map(_.name))
      val filteredFiles = files.filterNot(x => ignoreSet.contains(x.name)).toList

      if (filteredFiles.isEmpty) return

      this.semaphoreLock(con)
      try {
        for (k <- filteredFiles) {
          this.migrate(con, k.queries)
          this.insertMigrationHistory(con, k.name, k.queries);
        }
      } finally {
        this.semaphoreUnlock(con)
      }
    } finally {
      con.close()
    }
  }

  private def createTables(con: Connection): Unit = {
    val sql = """
     create table if not exists migration_histories (
      filename varchar(255) NOT NULL,
      queries text NOT NULL,
      executed_at timestamp without time zone default CURRENT_TIMESTAMP,
      primary key (filename)
    );
    create table if not exists migration_semaphores (
      username varchar(255) NOT NULL,
      executed_at timestamp without time zone default CURRENT_TIMESTAMP,
      primary key (username)
    )
    """

    for (q <- sql.split(";")) {
      val ps = con.prepareStatement(q)
      try {
        ps.execute()
      } finally {
        ps.close()
      }
    }
  }

  private def loadMigrationFiles(dir: String): List[MigrationFile] = {
    val path = Paths.get(dir)
    val files = Files.list(path).toList().asScala

    val buffer = ListBuffer.empty[MigrationFile]
    for (f <- files) {
      val name = f.getFileName.toString
      val queries = Files.readString(f)
      buffer += MigrationFile(name, queries)
    }

    return buffer.toList
  }

  private def fetchMigrationHistories(con: Connection): List[MigrationHistory] = {
    val sql = """
      select filename, executed_at from migration_histories
    """
    val ps = con.prepareStatement(sql)
    val rs = ps.executeQuery()

    try {
      val buffer = ListBuffer.empty[MigrationHistory]

      while (rs.next) {
        val h = MigrationHistory(
          rs.getString("name"),
          rs.getTimestamp("executed_at").toLocalDateTime
        );
        buffer += h
      }

      return buffer.toList
    } finally {
      rs.close()
      ps.close()
    }
  }

  private def migrate(con: Connection, queries: String): Unit = {
    val ps = con.prepareStatement(queries)

    try {
      ps.execute()
    } finally {
      ps.close()
    }
  }

  private def insertMigrationHistory(con: Connection, name: String, queries: String): Unit = {
    val sql = """
      insert into migration_histories (filename, queries) values (?, ?)
    """
    val ps = con.prepareStatement(sql)
    ps.setString(1, name)
    ps.setString(2, queries)

    try {
      ps.executeUpdate()
    } finally {
      ps.close()
    }
  }

  private def semaphoreLock(con: Connection): Unit = {
    val sql = """
      insert into migration_semaphores (username) values ('migrator')
    """
    val ps = con.prepareStatement(sql)

    try {
      ps.executeUpdate()
    } finally {
      ps.close()
    }
  }

  private def semaphoreUnlock(con: Connection): Unit = {
    val sql = """
      delete from migration_semaphores where username = 'migrator'
    """
    val ps = con.prepareStatement(sql)

    try {
      ps.executeUpdate()
    } finally {
      ps.close()
    }
  }
}

case class PostgresOptions(
    val jdbcUrl: String,
    val username: String,
    val password: String
)

case class MigrationFile(
    val name: String,
    val queries: String
)

case class MigrationHistory(
    val name: String,
    val executedAt: LocalDateTime
)
