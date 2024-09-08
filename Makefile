EXE = sacre_dieu

ifeq ($(OS),Windows_NT)
	override EXE := $(EXE).exe
endif

$(EXE):
	cargo +nightly rustc --release -- -C target-cpu=native --emit link=$(EXE)

clean:
	rm -f $(EXE)