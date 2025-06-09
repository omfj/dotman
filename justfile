all: build install

build:
    cargo build --release

install:
    sudo cp target/release/dotman /usr/local/bin/dotman

uninstall:
    sudo rm /usr/local/bin/dotman

test:
    cargo test