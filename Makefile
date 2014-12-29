all: server client

server:
	mkdir -p bin
	rustc -o bin/server -L ./target/deps src/server.rs --cfg server --extern time=target/deps/libtime-b2f52bc77c16c052.rlib

client:
	mkdir -p bin
	rustc -o bin/client --cfg client -L ./target/deps src/client.rs --extern time=target/deps/libtime-b2f52bc77c16c052.rlib

run: server client
	sh run_game.sh

clean:
	rm -rf bin
