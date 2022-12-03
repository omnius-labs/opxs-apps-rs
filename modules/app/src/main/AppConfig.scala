package omnius.pxtv.app

case class AppConfig(httpServer: HttpServerConfig, database: DatabaseConfig)

case class HttpServerConfig(host: String, port: Int)

case class DatabaseConfig(jdbcUrl: String, username: String, password: String)
