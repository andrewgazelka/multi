# `multi`

_Making parallel bash commands elegant ✨._

Suppose we want to run the two commands

```bash
run_unit_tests
run_integration_tests
```

We can instead re-write this as

```
multi -c "run_unit_tests" -c "run_integration_tests"
```

To the end-user, nothing will occur differenly.
However, the commands are actually executed in parallel.
The order of the `stdout` and `stderr` of the commands are preserved and an error in one command will stop other
commands. This is similar to the principal of [Structured Concurrency](https://vorpus.org/blog/notes-on-structured-concurrency-or-go-statement-considered-harmful/).
