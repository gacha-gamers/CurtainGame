# Tracy

In order to run Tracy, you first need to [obtain Tracy](https://github.com/wolfpld/tracy/releases).

Once you have finished downloading and extracting Tracy, run a terminal located at your tracy instalation directory:
```sh
$ cd /path/to/tracyinstallation/
$ capture -o my_capture.tracy
```
In another terminal, in the root of your project, execute
```sh
$ cargo run --release --features bevy/trace_tracy
``` 
Once the game is closed, a `my_capture.tracy` file will be generated in the tracy folder. Running the Tracy GUI (Tracy.exe on Windows), you should be able to open up that .tracy file to analyze the profiled data. 


# Flamegraph

## Getting the profiling folder set up
The `profiling` folder is where the cargo-flamegraph file must be located. If you have a release of flamegraph, you may place it there. However, it is recommended to build flamegraph from master for now, until a release that includes [this recent commit](https://github.com/flamegraph-rs/flamegraph/commit/5dbce355aef63b1e272625887e0cc806afdc186d) is made.

Here's the instructions for doing so (on Windows, for other OS it should be similar):


1. In the root project directory (together with Cargo.toml), create the profiling directory.
```sh
$ mkdir profiling 
``` 
2. Clone the flamegraph repo (latest)
```sh
$ cd profiling
$ git clone https://github.com/flamegraph-rs/flamegraph
``` 
3. Build the flamegraph repository
```sh
$ cd flamegraph
$ cargo build
```
4. Move the cargo-flamegraph.exe file generated from building the project to `/profiling`
```sh
$ cd target/debug
$ move cargo-flamegraph.exe ../../../
``` 
5. Delete the leftover flamegraph directory (only the cargo-flamegraph.exe file is necessary)
```
$ cd ../../../
$ rmdir /s flamegraph
``` 
---

## Executing flamegraph


In the project root directory, execute the command in an elevated terminal:
```sh
$ ./profiling/cargo-flamegraph flamegraph -c "record -g"
```

This will run flamegraph and leave behind a file called `flamegraph.svg`, which should be opened on a web browser for interactive visualization of the profiled data.