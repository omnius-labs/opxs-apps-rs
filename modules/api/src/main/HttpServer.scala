package omnius.pxtv.api

import scala.concurrent.Await
import scala.concurrent.ExecutionContext
import scala.concurrent.Future
import scala.concurrent.duration._
import scala.io.StdIn
import scala.util.Failure
import scala.util.Success

import akka.actor.typed.ActorSystem
import akka.actor.typed.scaladsl.AskPattern.Askable
import akka.actor.typed.scaladsl.AskPattern.schedulerFromActorSystem
import akka.actor.typed.scaladsl.Behaviors
import akka.http.scaladsl.Http
import akka.http.scaladsl.marshallers.sprayjson.SprayJsonSupport._
import akka.http.scaladsl.model._
import akka.http.scaladsl.server.Directives._
import akka.http.scaladsl.server.Route
import akka.util.Timeout
import spray.json.DefaultJsonProtocol._
import spray.json.RootJsonFormat
import ch.megard.akka.http.cors.scaladsl.CorsDirectives._

import omnius.pxtv.api.actors.CounterActor

object HttpServer {
  def main(args: Array[String]): Unit = {
    implicit val system: ActorSystem[CounterActor.Command] = ActorSystem(CounterActor(), "http-typed-counter")
    implicit val executionContext: ExecutionContext = system.executionContext

    val route = cors() {
      pathPrefix("counter") {
        concat(
          path("reset") {
            post {
              system ! CounterActor.Command.Reset
              complete(StatusCodes.Accepted, "reset")
            }
          },
          path("increment") {
            post {
              implicit val incrementFormat: RootJsonFormat[CounterActor.Command.Increase] =
                jsonFormat1(CounterActor.Command.Increase.apply)
              entity(as[CounterActor.Command.Increase]) { command => // place a bid, fire-and-forget
                system ! command
                complete(StatusCodes.Accepted, "increment")
              }
            }
          },
          path("decrement") {
            post {
              implicit val decrementFormat: RootJsonFormat[CounterActor.Command.Decrease] =
                jsonFormat1(CounterActor.Command.Decrease.apply)
              entity(as[CounterActor.Command.Decrease]) { command => // place a bid, fire-and-forget
                system ! command
                complete(StatusCodes.Accepted, "decrement")
              }
            }
          },
          get {
            implicit val counterResponseFormat: RootJsonFormat[CounterActor.Response.Count] =
              jsonFormat1(CounterActor.Response.Count.apply)
            implicit val timeout: Timeout = 5.seconds

            val getCount: Future[CounterActor.Response.Count] =
              system.ask(CounterActor.Command.ReadCount.apply).mapTo[CounterActor.Response.Count]
            complete(getCount)
          }
        )
      }
    }

    val bindingFuture: Future[Http.ServerBinding] =
      Http().newServerAt("0.0.0.0", 9080).bind(route)
    println(s"Server online at http://localhost:9080/\nPress RETURN to stop...")
    StdIn.readLine()
    bindingFuture
      .flatMap(_.unbind())
      .onComplete {
        case Success(value) =>
          println("finish")
          system.terminate()
        case Failure(exception) =>
          println("fail")
          system.terminate()
      }
  }
}
