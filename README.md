# `multi`

Run multiple commands faster and more simply.

Suppose we want to run the two commands

```bash
cargo clippy
cargo test
```

We can instead re-write this as

```
multi -c "cargo clippy" -c "cargo test"
```

To the end-user, nothing will occur differenly.
However, the commands are actually executed in parallel.
The order of the `stdout` and `stderr` of the commands are preserved and an error in one command will stop other
commands. This is similar to the principal of [Structured Concurrency](https://vorpus.org/blog/notes-on-structured-concurrency-or-go-statement-considered-harmful/).
