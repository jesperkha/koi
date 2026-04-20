# The main function

Similar to other C-like languages, the main function is the programs entry point. It returns an integer which is used as the programs exit code.

```go
func main() int {
    return 123
}
```

```sh
$ koi run
$ echo $?
123
```

Unlike C, the main functions signature *must* be `func() int`.
