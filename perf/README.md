# Perf tooling

| Lang | Library | Protocol | Requests | Avg | Total time |
| -- | -- | -- | -- |
| Rust | Actix | HTTP      | 1,000     | 0.2370ms | 267ms   |
| Rust | Actix | WebSocket | 1,000     | -        | 0.6ms   |
| Rust | Actix | Streaming | 1,000     | -        | 0.2ms   |
| Rust | Actix | HTTP      | 10,000    | 0.243ms  | 2,793ms |
| Rust | Actix | WebSocket | 10,000    | -        | 5.5ms   |
| Rust | Actix | Streaming | 10,000    | -        | 5.0ms   |
| Rust | Actix | WebSocket | 1,000,000 | -        | 471ms   |
| Rust | Actix | Streaming | 1,000,000 | -        | 515ms   |

| Java | Akka  | HTTP      | 1,000     | 0.852ms  | 891ms   |
| Java | Akka  | WebSocket | 1,000     | -        | 23ms    |
| Java | Akka  | Streaming | 1,000     | -        | 14ms    |
| Java | Akka  | HTTP      | 10,000    | -        | -       |
| Java | Akka  | WebSocket | 10,000    | -        | 195ms   |
| Java | Akka  | Streaming | 10,000    | -        | 35ms    |
| Java | Akka  | WebSocket | 1,000,000 | -        | 172,292ms   |
| Java | Akka  | Streaming | 1,000,000 | -        | 3088ms  |
