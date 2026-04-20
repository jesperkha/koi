# Project config

The `koi.toml` file contains the compiler parameters used when running `koi run` or `koi build`. The default configuration looks like this:

```
# Koi project configuration

[project]
name = "myApp"    # Project name
type = "app"      # Project type (app|package)
src = "src"       # Source code directory
bin = "bin"       # Output directory for temporary files
out = "."         # Output directory of targets
ignore-dirs = []  # Source directories to ignore
target = "x86-64" # Target arch (x86-64)
link-with = []    # Additional libraries to link with

[options]
debug-mode = false
```

You can override any of the `[project]` options by passing them as a flag:

```
$ koi build --out=dist --name=release
```

