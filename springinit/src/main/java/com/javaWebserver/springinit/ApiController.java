package com.javaWebserver.springinit;

import java.util.UUID;

import org.springframework.http.HttpStatus;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.servlet.mvc.method.annotation.StreamingResponseBody;

import com.fasterxml.jackson.databind.MapperFeature;
import com.fasterxml.jackson.databind.ObjectMapper;

@RestController
public class ApiController {
  private static final ObjectMapper defaultObjectMapper = new ObjectMapper().enable(MapperFeature.SORT_PROPERTIES_ALPHABETICALLY);

  @GetMapping(value = "/api/json", produces = MediaType.APPLICATION_JSON_VALUE)
	public ResponseEntity<JsonObj> apiJson() {
    return new ResponseEntity<JsonObj>(JsonObj.random(), HttpStatus.OK);
	}

  @GetMapping(value = "/stream", produces = MediaType.APPLICATION_STREAM_JSON_VALUE)
	public ResponseEntity<StreamingResponseBody> streaming(@RequestParam("times") String times) {
    int num_times = Integer.parseInt(times);
    System.out.println("Streaming "+ num_times + " items");

    StreamingResponseBody responseBody = response -> {
      response.write("[".getBytes());
      response.flush();
      for (int i = 0; i < num_times; i++) {
        if (i != 0) response.write(",".getBytes());

        response.write(
          defaultObjectMapper.writeValueAsString(
            defaultObjectMapper.createObjectNode().put("name", UUID.randomUUID().toString())
          ).getBytes()
        );

        response.flush();
      }
      response.write("]".getBytes());
      response.flush();
    };

    return ResponseEntity.ok().contentType(MediaType.APPLICATION_STREAM_JSON).body(responseBody);
	}
}
