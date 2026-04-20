# The Koi Programming Language

Koi is a very simple systems language. Both the modern syntax and module system makes low level development more enjoyable. Koi compiles to native x86-64 assembly on Linux and links directly with C libraries.

[View on GitHub](https://github.com/jesperkha/koi)

## Install

### Download latest release

This script downloads and installs Koi to `$HOME/.local/koi`. Make sure `$HOME/.local/koi/bin` is in your `$PATH`.

```sh
curl -sL https://raw.githubusercontent.com/jesperkha/koi/main/setup.sh | bash
```

### Or build from source

The install script builds the Koi binary along with the standard library and creates the installation directory `$HOME/.local/koi`. Make sure `$HOME/.local/koi/bin` is in your `$PATH`.

```sh
git clone https://github.com/jesperkha/koi.git
cd koi
./install.sh
```
