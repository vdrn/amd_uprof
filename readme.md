# amd_uprof

Safe wrapper for AMD uProf

Uses [amd_uprof-sys](https://github.com/vdrn/amd_uprof-sys) for FFI. If you have issues with build, refer to [amd_uprof-sys readme](https://github.com/vdrn/amd_uprof-sys)

## Usage

### Prerequisites
- If using CLI, add `-start-paused` option.
- if using GUI, make sure `Enable start paused` option is enabled.
- To enable the API, you'll need to call `amd_prof::enable(true)` once at the beggining of the program.

### Basic Usage
``` rust

amd_uprof::enable(true);  // once at the start

// ...

// start profiling
amd_uprof::resume_profiler();

// do some work

// stop profiling 
amd_uprof::pause_profiler();
```

Since `resume_profiler` and `pause_profiler` have very large overhead, there are also async versions that do not block: 
- `resume_profiler_async`
- `pause_profiler_async`

Event gathering will start/stop at some unspecified time after they are called.


### Task Scopes

To use Tasks (unfortunately UProf GUI does not display them, so they are only useful with CLI):

``` rust
amd_uprof::enable(true);  // once at the start

// ...

// start profiling
amd_uprof::resume_profiler();

{
 let _task_scope =  amd_prof::scope("domain", "name");
 
 // do some work

 // scope will be automaticall closed
}

```

Line number and file name (`nightly` is enabled) will be automatically assigned to the beggining of the task, but not the end. If you need that, you'll have to manually drop the scope using `finish()` method:

``` rust
amd_uprof::enable(true);  // once at the start

// ...

// start profiling
amd_uprof::resume_profiler();

let task_scope = amd_prof::scope("domain", "name");

// do some work

task_scope.finish();
```


## Features
- `nightly`: Needed for associating file name with the task.
