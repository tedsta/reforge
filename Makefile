all: server client

server:
	mkdir -p bin
	rustc -o bin/server -L ./lib src/server.rs --cfg server

client:
	mkdir -p bin
	rustc -o bin/client --cfg client -L ./lib src/client.rs -C link-args='-lcsfml-system -lcsfml-window -lcsfml-graphics -lcsfml-audio -lcsfml-network'

run: server client
	sh run_game.sh

clean:
	rm -rf bin
