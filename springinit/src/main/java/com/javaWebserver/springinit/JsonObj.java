package com.javaWebserver.springinit;

import java.util.UUID;

import com.fasterxml.jackson.annotation.JsonFormat;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonFormat
public class JsonObj {
  @JsonProperty("name")
  public String name;

  public JsonObj(String name) {
    this.name = name;
  }

  public static JsonObj random() {
    return new JsonObj(UUID.randomUUID().toString());
  }
}