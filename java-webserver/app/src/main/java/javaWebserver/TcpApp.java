package javaWebserver;

import static java.nio.ByteOrder.LITTLE_ENDIAN;

import java.nio.ByteBuffer;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletionStage;

import akka.Done;
import akka.NotUsed;
import akka.actor.typed.ActorRef;
import akka.actor.typed.ActorSystem;
import akka.actor.typed.Behavior;
import akka.actor.typed.javadsl.AbstractBehavior;
import akka.actor.typed.javadsl.ActorContext;
import akka.actor.typed.javadsl.Behaviors;
import akka.actor.typed.javadsl.Receive;
import akka.japi.Pair;
import akka.stream.OverflowStrategy;
import akka.stream.javadsl.Flow;
import akka.stream.javadsl.Sink;
import akka.stream.javadsl.Source;
import akka.stream.javadsl.Tcp;
import akka.stream.javadsl.Tcp.IncomingConnection;
import akka.stream.javadsl.Tcp.ServerBinding;
import akka.stream.typed.javadsl.ActorSource;
import akka.util.ByteString;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;

public class TcpApp extends AbstractBehavior<TcpApp.Command> {

  public static interface Command {}

  public static Behavior<Command> create() {
    return Behaviors.setup(TcpApp::new);
  }

  public TcpApp(ActorContext<Command> context) {
    super(context);

    startTcpServer();
  }

  @Override
  public Receive<Command> createReceive() {
    return newReceiveBuilder().build();
  }

  void startTcpServer() {
    ActorSystem<Void> system = this.getContext().getSystem();

    System.out.println("Starting TCP...");

    final Source<IncomingConnection, CompletionStage<ServerBinding>> connections = Tcp.get(system).bind("127.0.0.1", 3001);

    connections.runForeach(
      connection -> {
        System.out.println("New connection from: " + connection.remoteAddress());

        Source<ByteString, ActorRef<ByteString>> messageStream = ActorSource.<ByteString>actorRef(
          m -> false,
          m -> Optional.empty(),
          1_024_000,
          OverflowStrategy.fail()
        );

        Pair<ActorRef<ByteString>, Source<ByteString, NotUsed>> pair = messageStream.preMaterialize(system);

        Sink<ByteString, CompletionStage<Done>> sink = Sink.foreach(item -> {
          ByteBuf in = convertToByteBuffer(item);
          long num_times = in.readUnsignedIntLE();
          System.out.println("Send "+ num_times + " messages");

          ByteBuf out = Unpooled.buffer().order(LITTLE_ENDIAN);
          for (int i = 0; i < num_times; i++) {
            Helper.writeString(out, UUID.randomUUID().toString());
          }
          pair.first().tell(ByteString.fromByteBuffer(out.nioBuffer()));
        });

        Flow<ByteString, ByteString, NotUsed> flow = Flow.fromSinkAndSource(sink, pair.second());

        connection.handleWith(flow, system);
      },
      system
    );
  }

  private ByteBuf convertToByteBuffer(ByteString text) {
    ByteBuffer wrongEndian = text.asByteBuffer();
    ByteBuffer input = wrongEndian.order(LITTLE_ENDIAN);
    return Unpooled.buffer().order(LITTLE_ENDIAN).writeBytes(input);
  }
}
