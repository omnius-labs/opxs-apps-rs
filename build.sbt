lazy val commonSettings = Seq(
  organization := "com.omnius.pxtv",
  version := "1.0.0",
  scalaVersion := "3.2.0",
  scalacOptions ++= Seq("-noindent", "-rewrite"),
  fork := true,
  Compile / scalaSource := (Compile / sourceDirectory).value,
  Test / scalaSource := (Test / sourceDirectory).value,
  libraryDependencies ++= {
    val akkaVersion = "2.7.0"
    val akkaHttpVersion = "10.2.10"
    val circeVersion = "0.14.3"
    val testcontainersScalaVersion = "0.40.11"
    Seq(
      "com.typesafe.akka" %% "akka-actor-typed" % akkaVersion,
      "com.typesafe.akka" %% "akka-stream-typed" % akkaVersion,
      "com.typesafe.akka" %% "akka-persistence-typed" % akkaVersion,
      "com.typesafe.akka" %% "akka-persistence-query" % akkaVersion,
      "com.typesafe.akka" %% "akka-cluster-tools" % akkaVersion,
      "com.typesafe.akka" %% "akka-cluster-typed" % akkaVersion,
      "com.typesafe.akka" %% "akka-cluster-sharding-typed" % akkaVersion,
      "com.typesafe.akka" %% "akka-slf4j" % akkaVersion,
      "com.typesafe.akka" %% "akka-testkit" % akkaVersion % Test,
      "com.typesafe.akka" %% "akka-actor-testkit-typed" % akkaVersion % Test,
      "com.typesafe.akka" %% "akka-multi-node-testkit" % akkaVersion % Test,
      "com.typesafe.akka" %% "akka-http" % akkaHttpVersion,
      "com.typesafe.akka" %% "akka-http-spray-json" % akkaHttpVersion,
      "com.typesafe.akka" %% "akka-http-testkit" % akkaHttpVersion % Test
    ).map(_.cross(CrossVersion.for3Use2_13)) ++ Seq(
      "io.circe" %% "circe-core" % circeVersion,
      "io.circe" %% "circe-generic" % circeVersion,
      "io.circe" %% "circe-parser" % circeVersion,
      "ch.megard" %% "akka-http-cors" % "1.1.3",
      "org.slf4j" % "slf4j-api" % "2.0.3",
      "org.postgresql" % "postgresql" % "42.5.0",
      "ch.qos.logback" % "logback-classic" % "1.4.4",
      "com.dimafeng" %% "testcontainers-scala-scalatest" % testcontainersScalaVersion % Test,
      "com.dimafeng" %% "testcontainers-scala-postgresql" % testcontainersScalaVersion % Test,
      "org.scalatest" %% "scalatest" % "3.2.14" % Test
    )
  }
)

lazy val migration = (project in file("modules/migration"))
  .settings(commonSettings)

lazy val api = (project in file("modules/api"))
  .settings(commonSettings)
  .dependsOn(migration)
