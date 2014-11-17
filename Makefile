all: server client

server:
	mkdir -p bin
	rustc -o bin/server -L ./target/deps src/server.rs --cfg server

client:
	mkdir -p bin
	rustc -o bin/client --cfg client -L ./target/deps src/client.rs

run: server client
	sh run_game.sh

clean:
	rm -rf bin
