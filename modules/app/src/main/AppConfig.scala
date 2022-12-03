package omnius.pxtv.app

case class AppConfig (httpServer: HttpServerConfig)

case class HttpServerConfig(host: String, port: Int)
