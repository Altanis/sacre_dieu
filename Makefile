EXE = sacre_dieu

ifeq ($(OS),Windows_NT)
	override EXE := $(EXE).exe
endif

$(EXE):
	cargo rustc --release -- -C target-cpu=native --emit link=$(EXE)