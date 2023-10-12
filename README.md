



<img src="https://github.com/Software-Knife-and-Tool/thorn/blob/main/.github/thorn-eth.png?raw=true" width="20%" height="%20">

# *thorn/eth* - a system programming environment


### Under heavy development

*thorn/eth* is a LISP-idiomatic functionally-oriented interactive environment for system programming in the Rust ecosystem. It is targeted to low-resource persistent POSIX environments.

*thorn* is a LISP-1 namespaced programming language with Common Lisp idioms and macro system.

While *thorn* has much in common with Scheme, it is meant to be familiar to traditional LISP programmers.

*thorn* is a 2-Lisp, which gives it flexibility in implementation and subsequent customization. A small, native code runtime kernel supports system classes, function application, heap/system image management, and the FFI framework.

Subsequent layers based on the runtime offer advanced features.



#### Rationale

------

Functional languages bring us closer to a time where we can automatically prove our programs are correct. As systems get more complex, we'll need increased assurance that their various components are reliable. Modern programming concepts do much to underpin reliability and transparency.

*thorn* attempts to express modern programming concepts with a simple, familiar syntax. The venerable Common Lisp macro system helps the system designer create domain specific languages.

*LISPs* are intentionally dynamic which has selected against them for use in production environments, yet statically-typed languages produce systems that are hard to interact with and impossible to change *in situ*. Few languages in use today have adequate meta-programming facilities. We need systems that can we reason about and can supplement themselves.

Current systems tend to be large and resource-hungry. We need lean systems that can do useful work in low resource environments and are flexible enough to evolve to meet new demands. Current systems have runtimes measured in days, if for no other reason than improving them requires a complete reinstall. An evolving system can have a runtime measured in months or years.

Evolutionary response to change is the only defense a system has against obsolescence.

Most of our core computational frameworks are built on static systems and are fragile with respect to change. Such systems tend to be disposable. Lightweight dynamic systems designed for persistence are the next step.



#### Project Goals

------

- *thorn*, a functional forward system language
- *thorn/mu*, minimal POSIX runtime suitable for containers
- *thorn/eth*, a native code compiler
- small and simple installation, no external dependencies
- add interactivity and extensibility to application implementations
- Rust FFI system
- mostly Common Lisp semantics
- resource overhead equivalent to a UNIX shell
- minimal external crate dependencies
- futures multi-threading and non-blocking I/O



#### State of the *thorn* system

------

*thorn/eth* is a work in progress.

*thorn/mu* should build with rust 1.68 or better. *thorn/mu* builds are targeted to:

- x86-64 and AArch-64 Linux distributions
- x86-64 and M1 MacOs X
- x86-64 WSL
- Docker Ubuntu and Alpine containers

Portability, libraries, deployment, documentation, and garbage collection are currently the top priorities.



#### About *thorn*

------

*thorn* is an immutable, namespaced LISP-1 that borrows heavily from *Scheme*, but is more closely related to the Common LISP family of languages. *thorn* syntax and constructs will be familiar to the traditional LISP programmer. 

*thorn* leans heavily on functional programming principles.

The *thorn/mu* runtime kernel is written in mostly-safe `rust` (the system image/heap facility *mmaps* a file, which is an inherently unsafe operation.)

The runtime implements 64 bit tagged pointers, is available as a crate, extends a Rust API for embedded applications, and is an evaluator for the *thorn/mu* kernel language. *thorn/mu* provides the usual fixed-width numeric types, lists, fixed-arity lambdas, simple structs, LISP-1 symbol namespaces, streams, and specialized vectors in a garbage collected environment.

The *thorn* 2-LISP system is organized as a stack of compilers, culminating in the *thorn-eth* native code compiler/system builder.

The *core* library provides *rest* lambdas, closures, expanded struct types, *defun/defconst/defmacro* and a reader/compiler for those forms.

The *preface* library extends *core* with various lexical binding forms, *cond/and/or/progn*, and a library loading facility.

Optional libraries provide a variety of enhancements and services, including Common LISP macros and binding special forms.



#### Viewing the documentation

------

A handy ```thorn/mu``` reference card can be found in ```doc/refcards``` in a variety of formats.

The `thorn/mu` crate rustdoc documentation can be generated by

```
% make doc
```

and will end up in ```doc/rustdoc```. The ``doc/rustdoc/mu``  subdirectory contains the starting ```index.html```.

The *thorn* reference documentation is a collection of *markdown* files in `doc/reference`. To generate the documentation, you will need the *pandoc* utility, see *https://pandoc.org*

Once built, the *html* for the *reference* material will be in  *doc/reference/html*, starting with *index.html*.



#### Building and installing the *thorn* system

------

The *thorn* runtime *libmu* is a native code program that must be built for the target CPU architecture. The *thorn* build system requires only a `rust` compiler,`rust-fmt`,`clippy` and some form of the `make` utility. Other tools like  `valgrind` are optional.

Tests and performance measurement requires some version of `python 3`.

```
git clone https://github.com/Software-Knife-and-Tool/thorn.git
```

After cloning the *thorn* repository, the *rust* system can be built and installed with the supplied makefile.

```
% make release
% make debug
```

Having built the distribution, install it in `/opt/thorn`.

```
% sudo make install
```

Related build targets, `debug` and `profile`, compile for debugging and profiling respectively.`make` with no arguments prints the available targets.

If you want to repackage *thorn* after a change to the library sources:

```
% make dist
```

and then reinstall.

Note: the installation mechanism does not remove the installation directory before writing it and changes to directory structure and files will tend to accrete. The make uninstall target will remove that if desired.

```
% sudo make uninstall
```



#### Testing

------

The distribution includes a test suite, which should be run after every interesting change. The test suite consists of a several hundred individual tests roughly separated by namespace.

Failures in the *mu* tests are almost guaranteed to cause complete failure of subsequent tests.

```
% make tests/summary
% make tests/commit
```

The `summary` target produces a human readable test report. This summary will be checked into the repo at the next commit.

 The `commit` target will produce a diff between the current summary and the repo summary.

The `tests` makefile has additional facilities for development. The `help` target will list them.

```
% make -C tests help

--- test options
    cargo - run rust tests
    namespaces - list namespaces
    commit - create test summary
    tests - tests in $NS
    mu core - run all tests in namespace, raw output
    test - run single test in $NS/$TEST
    summary - run all tests in all namespaces and print summary
    
```



#### Performance metrics

------

Metrics include the average amount of time (in microsconds) taken for an individual test and the number of objects allocated by that test. Differences between runs in the same installation can be in the 10% range. Any changes in storage consumption or a large (10% or greater) increase in test timing warrant examination.

The **NTESTS** environment variable (defaults to 20) controls how many passes are included in a single test run.

On a modern Core I7 CPU at 3+ GHz, the default perf tests take approximately four minutes of elapsed time for both the *mu* and *core* namespaces.

```
% make -C perf base
% make -C perf current
% make -C perf diff
% make -C perf commit
```

The `base` target produces a performance run and establishes a base line. The `current`  target produces a secondary performance run. The current summary will be checked into the repo as the base at the next commit. The `diff` target produces a human-readable diff between `base` and `current`. 

The `perf` makefile has additional facilities for development, including reporting on individual tests. The`help` target will list them. 

In specific, a summary of significant performance changes (differences in measured resource consumption and/or a large difference in average test time between the current summary and the baseline summary.) Timing metrics are heavily CPU/clock speed dependent.

```
% make -C perf commit
```

produces a report of the differences between the current summary and the established baseline. The *commit* target reports on any change in storage consumption between the baseline and the current summary, and timing changes greater than 20% for any individual test. `commit` also establishes the `current` report as the new baseline in preparation for a commit to the repo.

For convenience, the *thorn* Makefile provides:

```
% make perf/base		# establish a baseline report
% make perf/current		# produce a secondary report
% make perf/diff		# diff baseline and current
% make perf/commit		# diff baseline and current, prepare for commit
```

The  `perf`  makefile offers some development options.

```
% make -C perf help

--- perf options
    namespaces - list namespaces
    list - tests in $NS
    $NS - run all tests in namespace, unformatted output
    base - run all tests in all namespaces, establish baseline report
    current - run all tests in all namespace, establish current report
    commit - compare current with base, promote current to base
    diff - compare current report with base report
    metrics - run tests and verbose report
    valgrind - run memcheck, callgrind, cachegrind reports

```



#### Running the *thorn* system

------

The *thorn* binaries, libraries, and source files are installed in `/opt/thorn`. The `bin` directory contains the binaries and shell scripts for running the system. A copy of the `mu` crate is included in `/opt/thorn/thorn` along with the `core` and `preface` library sources.

```
runtime			runtime binary, minimal repl
thorn			shell script for running the extended repl
```


```
OVERVIEW: runtime - posix platform thorn/mu
USAGE: runtime [options] [file...]

runtime: x.y.z: [-h?psvcelq] [file...]
OPTIONS:
  -h                   print this message
  -?                   print this message
  -v                   print version string and exit
  -p                   pipe mode, no repl
  -l SRCFILE           load SRCFILE in sequence
  -e SEXPR             evaluate SEXPR and print result
  -q SEXPR             evaluate SEXPR quietly
  -c name:value[,...]  environment configuration  	   
  [file ...]           load source file(s)
  
```

An interactive session for the extended *thorn* system is invoked by the`thorn` shell script, `:h` will print the currently available repl commands. Forms entered at the prompt are evaluated and the results printed. The prompt displays the current namespace.

```
% /opt/thorn/bin/thorn
;;; Thorn version 0.0.x (preface:repl) :h for help
user>
```

*rlwrap* makes the *thorn* and *runtime* repls much more useful, with command history and line editing.

```
% alias thorn='rlwrap -m /opt/thorn/bin/thorn'
```

Depending on your version of *rlwrap*, *thorn* may exhibit odd echoing behavior. Adding

```
set enable-bracketed-paste off
```



to your `~/.inputrc` may help.



