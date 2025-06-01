# Async Rust HTTP Server

Async Rust server written with Tokio. Use Express.js style handler functions to read and write headers and content.

## wrk load testing

Tested with `wrk` and some Linux/Intel machine. No `keep-alive` implemented.

```bash
➜  ~ wrk -t4 -c100 -d30s http://127.0.0.1:7878
Running 30s test @ http://127.0.0.1:7878
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     2.26ms    0.86ms  29.00ms   78.71%
    Req/Sec    10.39k   838.22    12.02k    81.08%
  1241108 requests in 30.04s, 303.00MB read
Requests/sec:  41316.71
Transfer/sec:     10.09MB
```
