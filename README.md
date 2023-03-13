# Socket throughput

Repo to compare relative throughput performance of HTTP requests, WebSocket messages & HTTP streaming.

## HTTP @ 1,000

| Lang | Library | Avg | Total time |
| -- | -- | -- | -- |
| Rust | Actix         | 0.104ms  | 113ms    |
| Java | Akka          | 0.395ms  | 410ms    |
| Java | Spring(tomcat)| 0.202ms  | 216ms    |
| Java | Spring(netty) | 0.296ms  | 307ms    |

## HTTP @ 10,000

| Lang | Library | Total time |
| -- | -- | -- |
| Rust | Actix         | 676ms    |
| Java | Akka          | 3,029ms  |
| Java | Spring(tomcat)| 928ms    |
| Java | Spring(netty) | 1,591ms  |

## WebSocket @ 1,000

| Lang | Library | Total time |
| -- | -- | -- |
| Rust | Actix          | 1.1ms    |
| Java | Akka           | 13ms     |
| Java | Spring(tomcat) | 8ms      |

## WebSocket @ 10,000

| Lang | Library | Total time |
| -- | -- | -- |
| Rust | Actix          | 6ms      |
| Java | Akka           | 50ms     |
| Java | Spring(tomcat) | 39ms     |

## WebSocket @ 1,000,000

| Lang | Library | Total time |
| -- | -- | -- |
| Rust | Actix          | 489ms    |
| Java | Akka           | 3,354ms  |
| Java | Spring(tomcat) | 3,315ms  |

## Streaming @ 1,000

| Lang | Library | Total time |
| -- | -- | -- |
| Rust | Actix          | 0.59ms   |
| Java | Akka           | 7ms      |
| Java | Spring(tomcat) | 6ms      |

## Streaming @ 10,000

| Lang | Library | Total time |
| -- | -- | -- |
| Rust | Actix          | 5ms      |
| Java | Akka           | 24ms     |
| Java | Spring(tomcat) | 49ms     |

## Streaming @ 1,000,000

| Lang | Library | Total time |
| -- | -- | -- |
| Rust | Actix          | 412ms    |
| Java | Akka           | 2,191ms  |
| Java | Spring(tomcat) | 3,829ms  |

## Socket @ 1,000

| Lang | Library | Total time |
| -- | -- | -- |
| Rust | Tokio          | 1.5ms |
| Java | Akka           | 2.8ms |
| Java | Spring(tomcat) | -     |
