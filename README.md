# LightC

**Light** **C**hat is a tiny server-client messaging protocol designed as part of a school project. Written entirely in Rust.

## Project goals

- [X] Designing a basic messaging protocol 
- [X] Implementing a client that supports the whole protocol 
- [X] mplementing a server that supports the whole protocol 
- [ ] Add support for multiple channels                      
- [ ] Add authorization for users                            
- [ ] Add moderation features                                

## Protocol specification

| Command    | Parameters           | Description                                                     |
| :---       | :---:                | ---                                                             |
|`CONNECT`   |`<username>`          | Connect to the server as `<username>`                           |
|`DISCONNECT`|                      | Disconnect from the server                                      |
|`SAY`       |`<message>`           | Send `<message>` to every user in the server                    |
|`NICK`      |`<new_username>`      | Change your username to `<new_username>`                        |

## Client goals

- [X] Parsing user input                                  
- [X] Supporting the whole protocol                       
- [X] Communicating with connected server                 
- [X] Outputting messages received from server            
- [X] Proper TUI                                          
- [ ] Config file support                                 
- [ ] Communicating with multiple channels/servers at once
