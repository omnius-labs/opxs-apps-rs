package omnius.pxtv.api.actors

import akka.actor.typed.ActorRef
import akka.actor.typed.Behavior
import akka.actor.typed.scaladsl.Behaviors

object CounterActor {
  private val InitialCount: Int = 0

  def apply(): Behavior[Command] = changeCount(InitialCount)

  private def changeCount(nextCount: Int): Behavior[Command] =
    counter(nextCount)

  private def counter(currentCount: Int): Behaviors.Receive[Command] =
    Behaviors.receiveMessage {
      case Command.Reset                => changeCount(InitialCount)
      case Command.Increase(value: Int) => changeCount(currentCount + value)
      case Command.Decrease(value: Int) => changeCount(currentCount - value)
      case Command.ReadCount(replyTo: ActorRef[Response.Count]) =>
        replyTo ! Response.Count(currentCount)
        Behaviors.same
    }

  sealed trait Command
  object Command {
    case object Reset extends Command
    case class Increase(value: Int) extends Command
    case class Decrease(value: Int) extends Command
    case class ReadCount(replyTo: ActorRef[Response.Count]) extends Command
  }

  sealed trait Response
  object Response {
    case class Count(value: Int) extends Response
  }
}
