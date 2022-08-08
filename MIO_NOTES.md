# mio (metal io) notes

TcpStream:
- if read returns Ok(0) either
    - the stream has been closed by the peer
    - the buffer has size of zero

Have:
- a thread which listens for new connections
    - when a connection is accepted
        - it is added to the event poll with relevant interests
        - it is added to a lookup table with its token

To read data from the connection:
- the token is added to the event poll Interest Writable
- events are polled and a event with the matching token returns true for is\_readable
- the event should return is_readable = true
- data is read from the connection
- if not enough data, reregister the token with the poll registry

To write data to the connection:

## preliminary performance results
- static http response
- no routing
- did parse http request

- worker count did not appear to change performance numbers
- measuring via localhost is pointless now since all data goes through loopback

- rerunning benchmarks results in significantly degraded performance or even no work being done

-- using
```
 RUST_LOG=debug RUSTFLAGS='-C target-cpu=native' cargo run --release --example mio_test
```

using 16 workers
```
 $ wrk -t12 -c500 -d10 --latency  http://127.0.0.1:8080/
Running 10s test @ http://127.0.0.1:8080/
  12 threads and 500 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    17.54ms   27.27ms 355.52ms   92.86%
    Req/Sec     1.39M   516.45k    2.63M    79.24%
  Latency Distribution
     50%   10.74ms
     75%   18.00ms
     90%   31.99ms
     99%  151.73ms
  36601572 requests in 10.04s, 61.36GB read
  Socket errors: connect 0, read 14254, write 139, timeout 0
Requests/sec: 3644782.78
Transfer/sec:      6.11GB
```

- rerunning benchmark without restarting
```
 $ wrk -t12 -c500 -d10 --latency  http://127.0.0.1:8080/
Running 10s test @ http://127.0.0.1:8080/
  12 threads and 500 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   278.97us    3.34ms  96.55ms   99.57%
    Req/Sec    98.57k   177.19k  457.85k    83.33%
  Latency Distribution
     50%   52.00us
     75%  111.00us
     90%  171.00us
     99%  273.00us
  59063 requests in 10.07s, 101.39MB read
  Socket errors: connect 0, read 39, write 2, timeout 0
Requests/sec:   5867.93
Transfer/sec:     10.07MB
```

- chat example using tokio
```
 $ wrk -t12 -c500 -d10 --latency  http://127.0.0.1:8080/
Running 10s test @ http://127.0.0.1:8080/
  12 threads and 500 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     2.54ms    9.56ms 228.16ms   96.66%
    Req/Sec    80.61k    13.01k  138.35k    77.60%
  Latency Distribution
     50%  226.00us
     75%    2.11ms
     90%    6.43ms
     99%   20.29ms
  9629181 requests in 10.10s, 16.14GB read
Requests/sec: 953472.81
Transfer/sec:      1.60GB
```

solution: reregister the server socket with poll
performance is significantly reduced from 3 million requests per second to 1 million / tokio performance
