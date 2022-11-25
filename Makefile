ifeq ($(OS), Windows_NT)
	NAME = ffk.exe
else ifeq ($(shell uname -s), Linux)
	NAME = ffk
else
	NAME = ffku
endif

all:
	rustc src/main.rs -o $(NAME)