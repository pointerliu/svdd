TOP_MODULE = compute_unit
SRCS = compute_unit.sv
CPP_SRC = tb.cpp
OUT_DIR = obj_dir

clean:
	rm -rf $(OUT_DIR)

$(OUT_DIR):
	mkdir -p $(OUT_DIR)

build: clean $(OUT_DIR)
	verilator --coverage-line -fno-table --timing --cc --exe \
		$(CPP_SRC) $(SRCS) \
		--build -o sim -CFLAGS "-std=c++17 -I obj_dir" && echo "build success"

run:
	./$(OUT_DIR)/sim

all: build run

.PHONY: clean build run all
