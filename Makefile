all: server client

server:
	mkdir -p bin
	rustc -o bin/server -L ./target/deps src/server.rs --cfg server --extern time=C:\Users\Teddy\Programming\Rust\spacegame-rust\target\deps/libtime-1d6301158a291dc6.rlib

client:
	mkdir -p bin
	rustc -o bin/client --cfg client -L ./target/deps src/client.rs --extern time=C:\Users\Teddy\Programming\Rust\spacegame-rust\target\deps/libtime-1d6301158a291dc6.rlib

run: server client
	sh run_game.sh

clean:
	rm -rf bin
