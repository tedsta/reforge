all: client

client:
	mkdir -p bin
	rustc -o bin/client -L ./lib src/client.rs

clean:
	rm -rf bin
