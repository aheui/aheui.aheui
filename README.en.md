# aheui.aheui

[한국어](README.md)

**Aheui.aheui** is an [Aheui](https://aheui.github.io/) interpreter written in Aheui itself. It is highly conforming one, passing every test in the [semi-official test suite](https://github.com/aheui/snippets/), and also a highly complex piece of Aheui code enough for testing Aheui implementations. [Rpaheui](https://github.com/aheui/rpaheui) is known to run aheui.aheui without a problem, and with its JIT capability this is the recommended implementation to run it.

Aheui.aheui has the following characteristics:

* The port (`ㅎ`) is not implemented and behaves like a stack.
* The available memory is about a half of the memory available to a *single* stack in the underlying implementation. If the implementation supports unbounded memory, so does aheui.aheui.
* The cell size is same to the underlying implementation.
* The maximum size of code depends on the underlying stack size, and is unbounded when the stack is also unbounded.
* The input and output encoding, numeric input format and error handling follows those of the underlying implementation.
* It pushes -1 on the end-of-file condition.
* The horizontal border varies per the line, and once the border is reached, it wraps to the first/last row/column regardless of the instruction pointer and direction. This matches with most existing implementations.

Aheui.aheui itself needs the underlying implementation satisfying those:

* It passes the entirety of `standard` test suite.
* It pushes -1 on the EOF condition and never reflects.

In order to distinguish the code and input, aheui.aheui also treats a zero byte (`\0`) as the end of code and reads the rest (may contain zero bytes) as the simulated code executes. `test.sh`, originally from the test suite, was modified to account for this. This can be used to work around otherwise conforming implementation too. Aheui.aheui *may* work on the implementation which may return non-negative code points past the EOF, but it has not been designed for such case.

It is able to run itself, making Aheui one of (surprisingly) a few esoteric programming languages with [self-interpreter](http://esolangs.org/wiki/EsoInterpreters). The interpreter is put in the public domain; see the [license](LICENSE.txt) for details.

## See also

* [`proto.rs`](proto.rs) is a [Rust](http://www.rust-lang.org/) prototype which simulates the core stack and queue design. It is believed that the final `aheui.aheui` is functionally equivalent to this program except for the error handling.
* [`aheui.aheuis`](aheui.aheuis) is an Rpaheui assembly used to produce the final code.
* [`test.sh`](test.sh) is a test runner for the prototype, assembly and final code. The latter two requires the `$RPAHEUI` environment variable.

