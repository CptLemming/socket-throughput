package com.javaWebserver.springinit;

import java.io.IOException;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.CopyOnWriteArrayList;

import org.springframework.stereotype.Component;
import org.springframework.web.socket.TextMessage;
import org.springframework.web.socket.WebSocketSession;
import org.springframework.web.socket.handler.TextWebSocketHandler;

import com.fasterxml.jackson.databind.MapperFeature;
import com.fasterxml.jackson.databind.ObjectMapper;

@Component
public class SocketHandler extends TextWebSocketHandler {

	List<WebSocketSession> sessions = new CopyOnWriteArrayList<>();
  private static final ObjectMapper defaultObjectMapper = new ObjectMapper().enable(MapperFeature.SORT_PROPERTIES_ALPHABETICALLY);

	@Override
	public void handleTextMessage(WebSocketSession session, TextMessage message) throws InterruptedException, IOException {
    System.out.println("Payload -> "+ message.getPayload());
    Integer num_times = Integer.parseInt(message.getPayload());

		for (WebSocketSession webSocketSession : sessions) {
			// Map value = new Gson().fromJson(message.getPayload(), Map.class);
			// webSocketSession.sendMessage(new TextMessage("Hello " + value.get("name") + " !"));
      for (int i = 0; i < num_times; i++) {
        // webSocketSession.sendMessage(new TextMessage("Hello, world"));
        webSocketSession.sendMessage(new TextMessage(
          defaultObjectMapper.writeValueAsString(
            defaultObjectMapper.createObjectNode().put("name", UUID.randomUUID().toString())
          )
        ));
      }
		}
	}

	@Override
	public void afterConnectionEstablished(WebSocketSession session) throws Exception {
    sessions.forEach(s -> {
      try {
        s.close();
      } catch (Exception e) {}
    });
    sessions.clear();
		sessions.add(session);
	}
}
