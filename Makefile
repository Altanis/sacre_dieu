EXE = sacre_dieu

ifeq ($(OS),Windows_NT)
    override EXE := $(EXE).exe
endif

all:
    cargo rustc --release -- -C target-cpu=native --emit link=$(EXE)