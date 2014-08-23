all: client

client:
	mkdir -p bin
	rustc -o bin/client -L ./lib src/client.rs -C link-args="-lcsfml-system -lcsfml-window -lcsfml-graphics -lcsfml-audio -lcsfml-network"

run: client
	bin/client

clean:
	rm -rf bin
