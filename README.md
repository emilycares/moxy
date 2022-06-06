# moxy
A proxy for developers. That can be used for frontend and backend.

Its main use case it to cache HTTP requests.

## Usecase
You want to call and API that does not exist or is currently changing, and you 
are the person that needs to integrate this new/changed API in your software.
So instead of calling your real backend, you can start moxy in between and call 
the same routes as before. Moxy will capture all requests and will save them on
the file system. 
Moxy will only create fetch from the backend once. You can modify the file to 
change the response. Or add a route in the moxy.json to test your app, like it
was the real backend behind it.

## Supported platforms
It is working on Linux and Windows 10. If you are running another operating 
system just try to run it. It will probably work. These are just the platforms
that were tested.

## Build from source
Install rust from https://www.rust-lang.org/.
``` bash
cargo build --release
```
