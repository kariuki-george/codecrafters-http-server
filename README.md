[![progress-banner](https://backend.codecrafters.io/progress/http-server/dbbe22ab-bc9b-4fba-be70-ac26a8bbab11)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own HTTP server" Challenge](https://app.codecrafters.io/courses/http-server/overview).

[HTTP](https://en.wikipedia.org/wiki/Hypertext_Transfer_Protocol) is the
protocol that powers the web. In this challenge, you'll build a HTTP/1.1 server
that is capable of serving multiple clients.

Along the way you'll learn about TCP servers,
[HTTP request syntax](https://www.w3.org/Protocols/rfc2616/rfc2616-sec5.html),
and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

# Implemented Features

- ✅ TCP Connection Bind on port of choice
- ✅ HTTP protocol parsing
  - Path variables
  - Headers
  - Body
  - Query parameters
- ✅ Request parsing and Response writing lifecycle
- ✅ Concurrency and parallel processing through multithreading
- ✅ A dynamic router implementation
  - Can handle dynamic path params e.g /product/:id
- ✅ Data compression with GZip
- ✅ Example on examples folder

# Get started

```bash
git clone https://github.com/kariuki-george/codecrafters-http-server.git
cd codecrafters-http-server
mkdir files
cargo run -- --directory ./files/
```

# Test

On another terminal

- Echo

```bash
curl http://localhost:4221/echo/hellofriend
```

- Get User-Agent

```bash
curl http://localhost:4221/echo/useragent
```

- Get Request Params

```bash
curl http://localhost:4221/query?page=1&pos=1&cur_pos=1&ads_per_page=20&ads_count=20
```

- Write into a file

```bash
curl --header "Content-Type: application/text"   --request POST   --data 'Hello from George🤭'   http://localhost:4221/files/user
```

- Read from a file

```bash
curl http://localhost:4221/files/user
```
