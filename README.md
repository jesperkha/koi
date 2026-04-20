<br />
<div align="center">
  <img src=".github/assets/logo.svg" alt="Logo" width="200">

  <p align="center">
    <b>The Koi programming language</b>
    <br>
    <a href="https://github.com/jesperkha/koi/releases/latest"><strong>Latest release »</strong></a>
    <br />
    <br />
  </p>
</div>

## About

Koi is a very simple systems language. Both the modern syntax and module system makes low level development more enjoyable. Koi compiles to native x86-64 assembly on Linux and links directly with C libraries.

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

## Documentation

For language documentation, [refer to the Koi Wiki](https://jesperkha.github.io/koi/).

