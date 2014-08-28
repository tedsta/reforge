all: server client

client:
	mkdir -p bin
	rustc -o bin/client -L ./lib src/client.rs -C link-args="-lcsfml-system -lcsfml-window -lcsfml-graphics -lcsfml-audio -lcsfml-network"

server:
	mkdir -p bin
	rustc -o bin/server src/server.rs

run: server client
	sh run_game.sh

clean:
	rm -rf bin