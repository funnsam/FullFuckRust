ifeq ($(OS), Windows_NT)
	NAME = ffk.exe
else ifeq ($(shell uname -s), Linux)
	NAME = ffk
else
	NAME = ffku
endif

all:
	rustc ./src/main.rs -o $(NAME)

windows:
	rustc ./src/main.rs -o ffk.exe --target x86_64-pc-windows-gnu

linux:
	rustc ./src/main.rs -o ffk --target x86_64-unknown-linux-gnu