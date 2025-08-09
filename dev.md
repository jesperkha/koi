# Dev

## Plan

Passes:

- Assemble Packages:

  - Load files, put into FileSet (read_dir()?)
  - Parser, using Scanner, outputs AST for each file in set
  - Create an interface for all exported symbols in ASTs from FileSet, outputs Exports
  - Put all ASTs and Exports into Package

- Resolve symbols and check:

  - Create Universe, add all package Exports, add all builtins
  - Run TypeChecker on each package using Universe, create SemanticTable

- Compile:

  - Generate IR file for each package using its SemanticTable
  - Generate ASM from each IR file
  - Compile and link
  - Profit
