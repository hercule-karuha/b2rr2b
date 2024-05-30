# Run adder example

```
// build the rlib
$ cd bluesim-rilb
$ cargo build

// make bluesim
$ cd ../examples/AdderPipeline/
$ make
```

In another terminal:

```
$ cd examples/adder_analysis/
$ cargo run
```

Then run the bluesim

```
$ make run
```



The Tensteage is similar, but the bluesim will stuck. So you need to close the bluesim and rust program manually.