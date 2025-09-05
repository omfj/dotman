all: build install

build:
    cargo build --release

install: build
    sudo cp target/release/dotman $HOME/.local/bin/dotman

uninstall:
    sudo rm /usr/local/bin/dotman

test:
    cargo test
