lazy val commonSettings = Seq(
  organization := "com.omnius.pxtv",
  version := "1.0.0",
  scalaVersion := "2.13.10",
  fork := true,
  Compile / scalaSource := (Compile / sourceDirectory).value,
  Test / scalaSource := (Test / sourceDirectory).value,
  Compile / resourceDirectory := baseDirectory.value / "conf",
  libraryDependencies ++= {
    val akkaVersion = "2.7.0"
    val akkaHttpVersion = "10.4.0"
    val circeVersion = "0.14.3"
    val testcontainersScalaVersion = "0.40.11"
    Seq(
      "com.typesafe.akka" %% "akka-actor-typed" % akkaVersion,
      "com.typesafe.akka" %% "akka-stream-typed" % akkaVersion,
      "com.typesafe.akka" %% "akka-http" % akkaHttpVersion,
      "com.typesafe.akka" %% "akka-http-spray-json" % akkaHttpVersion,
      "de.heikoseeberger" %% "akka-http-circe" % "1.39.2",
      "ch.megard" %% "akka-http-cors" % "1.1.3",
      "io.circe" %% "circe-core" % circeVersion,
      "io.circe" %% "circe-generic" % circeVersion,
      "io.circe" %% "circe-parser" % circeVersion,
      "org.slf4j" % "slf4j-api" % "2.0.4",
      "ch.qos.logback" % "logback-classic" % "1.4.5",
      "org.postgresql" % "postgresql" % "42.5.0",
      "com.github.pureconfig" %% "pureconfig" % "0.17.2",
      "com.dimafeng" %% "testcontainers-scala-scalatest" % testcontainersScalaVersion % Test,
      "com.dimafeng" %% "testcontainers-scala-postgresql" % testcontainersScalaVersion % Test,
      "com.typesafe.akka" %% "akka-testkit" % akkaVersion % Test,
      "com.typesafe.akka" %% "akka-actor-testkit-typed" % akkaVersion % Test,
      "com.typesafe.akka" %% "akka-http-testkit" % akkaHttpVersion % Test,
      "org.scalatest" %% "scalatest" % "3.2.14" % Test
    )
  }
)

lazy val migration = (project in file("modules/migration"))
  .settings(commonSettings)

lazy val app = (project in file("modules/app"))
  .settings(commonSettings)
  .dependsOn(migration)
