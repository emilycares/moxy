# moxy
A web proxy for developers that can be used for frontend and backend.

Its main use case is to cache HTTP requests.

## Get Started
1. Build
2. Copy executable to some directory
3. Start and stop it
4. Change the remote in moxy.json

## Usecase
You want to call an API that does not exist or is currently changing, and you 
are the person that needs to integrate this new/changed API in your software?
So instead of calling your real backend, you can start moxy in between and call 
the same routes as before. Moxy will capture all requests and save them to
the file system. 
Moxy will only create fetch from the backend once. You can modify the file to 
change the response. Or add a route in the moxy.json to test your app, like it
was the real backend behind it.

## Supported platforms
It is tested on Linux and Windows 10.

## Build from source
Install rust from https://www.rust-lang.org/.
``` bash
cargo build --release
```
