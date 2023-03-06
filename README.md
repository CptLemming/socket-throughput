# Socket throughput

Repo to compare relative throughput performance of HTTP requests, WebSocket messages & HTTP streaming.

| Lang | Library | Protocol | Requests | Avg | Total time |
| -- | -- | -- | -- | -- | -- |
| Rust | Actix | HTTP      | 1,000     | 0.104ms  | 113ms    |
| Rust | Actix | WebSocket | 1,000     | -        | 0.12ms   |
| Rust | Actix | Streaming | 1,000     | -        | 0.59ms   |
| Rust | Actix | HTTP      | 10,000    | -        | 676ms    |
| Rust | Actix | WebSocket | 10,000    | -        | 6ms      |
| Rust | Actix | Streaming | 10,000    | -        | 5ms      |
| Rust | Actix | WebSocket | 1,000,000 | -        | 489ms    |
| Rust | Actix | Streaming | 1,000,000 | -        | 412ms    |
| Java | Akka  | HTTP      | 1,000     | 0.395ms  | 410ms    |
| Java | Akka  | WebSocket | 1,000     | -        | 13ms     |
| Java | Akka  | Streaming | 1,000     | -        | 7ms      |
| Java | Akka  | HTTP      | 10,000    | -        | 3,029ms  |
| Java | Akka  | WebSocket | 10,000    | -        | 50ms     |
| Java | Akka  | Streaming | 10,000    | -        | 24ms     |
| Java | Akka  | WebSocket | 1,000,000 | -        | 3,354ms  |
| Java | Akka  | Streaming | 1,000,000 | -        | 2,191ms  |
