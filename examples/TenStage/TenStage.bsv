bsc -u -sim -bdir build -p .:%/Libraries:../../probe-blue/ -simdir build AdderPipeline.bsv 
bsc -sim -e mkAdderPipeline  -bdir build -simdir build -o hello.out ../../bulesim-rlib/target/debug/libblue.a