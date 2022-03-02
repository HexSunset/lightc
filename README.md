# LightC

***WORK IN PROGRESS*** Currently very unstable and may change at any time

**Light** **C**hat is a tiny server-client messaging protocol designed as part of a school project. Written entirely in Rust.

## Project goals

| Feature                                                | Progress |
| :---                                                   | :---:    |
| Designing a basic messaging protocol                   | `[X]`    |
| Implementing a client that supports the whole protocol | `[*]`    |
| Implementing a server that supports the whole protocol | `[ ]`    |
| Add support for multiple channels                      | `[ ]`    |
| Add authorization for users                            | `[ ]`    |
| Add moderation features                                | `[ ]`    |

## Protocol specification

| Command    | Parameters           | Description                                                     |
| :---       | :---:                | ---                                                             |
|`PING`      |                      | Sent by the server to check that the connection is still active |
|`PONG`      |                      | Sent by the client as a response to a `PING` command            |
|`CONNECT`   |`<username>`          | Connect to the server as `<username>`                           |
|`DISCONNECT`|                      | Disconnect from the server                                      |
|`SAY`       |`<message>`           | Send `<message>` to every user in the server                    |
|`WHISPER`   |`<message>` `<target>`| Send `<message>` only to user `<target>`                        | 

## Client goals

| Feature                                              | Progress |
| :---                                                 | :---:    |
| Parsing user input                                   | `[X]`    |
| Supporting the whole protocol                        | `[*]`    |
| Communicating with connected server                  | `[ ]`    |
| Outputting messages received from server             | `[ ]`    |
| Proper TUI                                           | `[ ]`    |
| Config file support                                  | `[ ]`    |
| Communicating with multiple channels/servers at once | `[ ]`    |
