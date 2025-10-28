# Quarter Runtime Library Makefile
# Compiles runtime.c to runtime.o for linking with compiled Forth programs

CC = cc
CFLAGS = -O2 -fPIC -Wall -Wextra

.PHONY: all clean

all: runtime.o

runtime.o: runtime.c
	$(CC) $(CFLAGS) -c runtime.c -o runtime.o

clean:
	rm -f runtime.o

# For development: compile with debug symbols
debug: CFLAGS = -g -fPIC -Wall -Wextra
debug: runtime.o
