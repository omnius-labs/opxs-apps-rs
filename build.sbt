ThisBuild / scalaVersion := "2.13.10"
ThisBuild / organization := "com.omnius.pxtv"
ThisBuild / version := "1.0.0"
ThisBuild / fork := true

lazy val defaultLibs = {
  val akkaVersion = "2.6.20"
  val testcontainersScalaVersion = "0.40.11"
  Seq(
    "com.typesafe.akka" %% "akka-actor-typed" % akkaVersion,
    "com.typesafe.akka" %% "akka-stream-typed" % akkaVersion,
    "org.slf4j" % "slf4j-api" % "2.0.4",
    "ch.qos.logback" % "logback-classic" % "1.4.5",
    "org.postgresql" % "postgresql" % "42.5.0",
    "com.github.pureconfig" %% "pureconfig" % "0.17.2",
    "com.dimafeng" %% "testcontainers-scala-scalatest" % testcontainersScalaVersion % Test,
    "com.dimafeng" %% "testcontainers-scala-postgresql" % testcontainersScalaVersion % Test,
    "com.typesafe.akka" %% "akka-testkit" % akkaVersion % Test,
    "com.typesafe.akka" %% "akka-actor-testkit-typed" % akkaVersion % Test,
    "org.scalatest" %% "scalatest" % "3.2.14" % Test
  )
}

lazy val migration = (project
  .in(file("modules/migration")))
  .settings(libraryDependencies ++= defaultLibs)

lazy val api = (project
  .in(file("modules/api")))
  .enablePlugins(PlayScala)
  .settings(
    libraryDependencies ++= defaultLibs ++ Seq(
      // https://stackoverflow.com/questions/67291502/play-framework-scala-hello-world-fail-on-ubuntu-20
      guice,
      "com.google.inject" % "guice" % "5.1.0",
      "com.google.inject.extensions" % "guice-assistedinject" % "5.1.0",
      "org.scalatestplus.play" %% "scalatestplus-play" % "5.0.0" % Test
    )
  )
  .dependsOn(migration)
