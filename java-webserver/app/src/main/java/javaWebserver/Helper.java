package javaWebserver;

import java.util.Arrays;

import io.netty.buffer.ByteBuf;

public class Helper {
  public static String readString(ByteBuf buf) {
    var length = buf.readUnsignedIntLE();
    StringBuilder sb = new StringBuilder();

    for (int i = 0; i < length; i++) {
      sb.append((char) buf.readByte());
    }

    return sb.toString();
  }

  public static void writeString(ByteBuf buf, String str) {
    buf.writeIntLE(str.length());
    buf.writeBytes(str.getBytes());
    // System.out.println("UUID -> "+ str);
    // System.out.println("LEN -> "+ str.length());
    // System.out.println("Bytes -> "+ Arrays.toString(str.getBytes()));
  }
}
