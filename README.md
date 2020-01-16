create project with "cargo new" then copy code to main and dependencies to Cargo.toml (threadpool = "1.0" rand = "0.7")

cargo build 

to run server - cargo run then port then 0.0.0.0 then nubmers of clients, for example: cargo run 8081 0.0.0.0 10

to run client - cargo run then port then ip then 0, for example: cargo run 8081 127.0.0.1 0

then you can type messages and run another clients
