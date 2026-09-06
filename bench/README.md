# Is it right yet?

The site's tagline says Xag has excellent runtime performance. That is a goal
rather than a measurement, and nothing on the site claims otherwise — the brief
rules performance claims off the pages entirely, because there are no published
numbers and there is nothing here worth quoting as one.

This is how to find out where the goal stands, so the answer is a command
rather than somebody's recollection.

```sh
./bench/run.py                     # finds xagc at the usual place
XAGC=/path/to/xagc ./bench/run.py  # or say where it is
```

It builds the same loop twice, once with `xagc build` and once with
`clang -O3`, runs each seven times after a warm-up, and refuses to report
anything if the two disagree about the answer.

## What the two cases are for

`add.xag` is the control: a billion iterations of arithmetic that compiles to
instructions. Both compilers fold it to a constant and never run the loop, which
is the point — it says the Xag backend is getting real optimisation, so whatever
the other case costs is about the operation rather than about codegen.

`loop.xag` is the measurement: the same billion iterations with a `mod` in the
body. `mod` is written rather than a plain series so that nothing can work the
answer out in advance instead of running the loop.

## Where it stood on 2026-09-06

```
loop body                        C -O3     Xag build    ratio
total + (i x 3)   native         3.9 ms       4.2 ms     1.1x
total + (i mod 7)              429.6 ms     578.0 ms     1.3x
```

Read the ratio and not the milliseconds. The absolute numbers move by a third
between runs depending on what else the machine is doing; the ratio holds.

## What happened earlier the same day

Measured a few hours before the above, the second row read **316 ms against
6543 ms — twenty times slower**. The cause was visible in `xagc ir`: `mod` was
not an instruction but a call out to the runtime, widened to `i128` and
truncated back, which the optimiser could not see through, so `mod 7` never
became the multiply-and-shift that clang emits and nothing vectorised.

```llvm
%1 = tail call i128 @xag_int_mod(i128 %0, i128 7, i32 64, i32 1)
```

The width and signedness in that call were already compile-time constants, so
the operation could be emitted inline instead. It now is:

```llvm
%0 = urem i32 %.lhs.trunc, 7
```

LLVM strength-reduces it and narrows it to 32 bits, having proved the range.
Twenty times became one and a bit, in an afternoon.

Which is the argument for keeping this here rather than writing a number down
somewhere. A benchmark from this morning was wrong by lunchtime, and it will be
wrong again — the useful thing is not the figure but being able to ask.
