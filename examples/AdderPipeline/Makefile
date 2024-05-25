BUILD_DIR = build
BLUESIM_RLIB = ../../bluesim-rlib/target/debug/libblue.a

all: adder.out

adder.out: | $(BUILD_DIR)
	bsc -u -sim -bdir $(BUILD_DIR) -p .:%/Libraries:../../probe-blue/ -simdir $(BUILD_DIR) AdderPipeline.bsv
	bsc -sim -e mkAdderPipeline -bdir $(BUILD_DIR) -simdir $(BUILD_DIR) -o adder.out $(BLUESIM_RLIB)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

run:
	B2R_SOCKET=/tmp/adder ./adder.out

clean:
	rm -rf $(BUILD_DIR) adder.*