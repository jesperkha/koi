# Koi Compiler — Workspace Crate Split Report

## Overview

This report proposes splitting the Koi compiler from a single monolithic package into a Cargo workspace of focused crates. The current structure is one package at `v0.10.0` with 13 logical modules, all living under `src/`. The goal is clean separation of concerns, faster incremental builds, and the ability to reuse components (e.g. the parser) independently of the full compiler.

The compiler pipeline today is:

```
Source files
  → Scanner (tokenize)
  → Parser (AST)
  → Dependency sort
  → Type checker
  → IR lowering
  → Code generation (C or x86-64)
  → Link
```

This maps naturally onto a layered set of crates with a strict one-way dependency chain and no circular dependencies.

---

## Proposed Workspace Structure

```
koi/                         ← workspace root (Cargo.toml with [workspace])
├── crates/
│   ├── koi-common/          ← errors, config, utilities
│   ├── koi-ast/             ← tokens, AST nodes, source map, module paths
│   ├── koi-scanner/         ← lexer (tokenizes source into tokens)
│   ├── koi-sema/            ← types, modules, symbols, context, imports
│   ├── koi-parser/          ← parser (tokens → AST), import validation, dep graph
│   ├── koi-ir/              ← intermediate representation
│   ├── koi-typecheck/       ← type checking (AST → TypedAST)
│   ├── koi-lower/           ← lowering (TypedAST → IR)
│   ├── koi-codegen-c/       ← C backend (IR → C source → compile via gcc)
│   ├── koi-codegen-x86/     ← x86-64 backend (IR → assembly → assemble via gcc)
│   └── koi-driver/          ← orchestrator; ties all phases together
└── src/                     ← binary entry point (koi CLI)
    ├── main.rs
    └── cmd.rs
```

The dependency graph is strictly acyclic:

```
koi (bin)
  └── koi-driver
        ├── koi-codegen-c, koi-codegen-x86
        │     └── koi-ir, koi-sema, koi-common
        ├── koi-lower
        │     └── koi-ir, koi-sema
        ├── koi-typecheck
        │     └── koi-sema, koi-ast
        ├── koi-parser
        │     └── koi-ast, koi-scanner, koi-common
        └── koi-common (config, error, util)
```

---

## Crate Proposals

---

### 1. `koi-common`

**Justification**

Today, `error.rs`, `config.rs`, and `util/` are imported by nearly every other module. They form a shared foundation with no dependencies on the rest of the compiler. Extracting them removes the need for every crate to depend on a giant monolith just to access a `Report` or `Config`.

**Contents**
- `error.rs` → `Diagnostics`, `Report`, `Res<T>`
- `config.rs` → `Config`, `ProjectType`, `Codegen`, `DriverPhase`
- `util/io.rs` → file I/O helpers
- `util/vartable.rs` → `VarTable` (used during IR lowering; can stay here or move to `koi-lower`)
- `util/testing.rs` → test helpers (gated with `#[cfg(test)]` or a `testing` feature flag)

**Public API**

```rust
// Error handling
pub type Res<T> = Result<T, Diagnostics>;
pub struct Diagnostics { ... }
pub struct Report { pub message: String, pub pos: Option<Pos>, ... }

// Configuration
pub struct Config { pub debug: bool, pub codegen: Codegen, ... }
pub enum Codegen { C, X86 }
pub enum ProjectType { App, Package }
pub enum DriverPhase { Parse, TypeCheck, Ir, Full }
```

**Changes needed**
- Move `src/error.rs`, `src/config.rs`, `src/util/` to `crates/koi-common/src/`
- `Pos` currently lives in `ast/token.rs` but is referenced by `error.rs` — either move `Pos` into `koi-common` or keep it in `koi-ast` and add `koi-ast` as a dependency of `koi-common`. The cleaner choice is to move `Pos` here since it is a primitive with no other AST dependencies.

---

### 2. `koi-ast`

**Justification**

The AST crate is a pure data layer — it defines the shape of parsed source code. It has no business logic; it is consumed by the parser, type checker, and lowering phase. Separating it means downstream crates can declare types without pulling in parsing or scanning logic.

`ImportPath` and `ModulePath` also live here rather than in `koi-sema`. See the **Co-dependency note** below for why.

**Contents**
- `ast/nodes.rs` → `Decl`, `Stmt`, `Expr`, `TypeNode`, `Literal`, `ImportNode`
- `ast/token.rs` → `Token`, `TokenKind`, `Pos`
- `ast/source.rs` → `Source`, `SourceMap`, `SourceId`
- `ast/file.rs` → `File`, `FileSet`, `Import`, `Ast`
- `ast/print.rs` → AST pretty-printer (debug only)
- `module/path.rs` → `ImportPath`, `ModulePath` (moved from `koi-sema`)

**Public API**

```rust
// Core node types
pub enum Decl { ... }
pub enum Stmt { ... }
pub enum Expr { ... }
pub struct Ast { pub imports: Vec<ImportNode>, pub decls: Vec<Decl> }

// Source tracking
pub struct SourceMap { ... }
pub struct Source { pub id: SourceId, pub content: String, pub path: PathBuf }
pub struct Pos { pub row: u32, pub col: u32, pub source: SourceId }

// Compilation unit
pub struct File { pub ast: Ast, pub source: Source }
pub struct FileSet { pub modpath: ModulePath, pub imports: HashSet<Import>, pub files: Vec<File> }

// Module paths (syntactic; derived directly from import tokens)
pub struct ImportPath { ... }
pub struct ModulePath { pub prefix: String, pub package: String, pub path: String }
impl From<&ImportNode> for ImportPath { ... }
impl From<ImportPath> for ModulePath { ... }
```

**Changes needed**
- Move `src/ast/` to `crates/koi-ast/src/`
- Move `src/module/path.rs` into `crates/koi-ast/src/` as `ast/path.rs` (or a top-level module)
- `koi-sema` imports `ImportPath` and `ModulePath` from `koi-ast` — no change to `koi-sema`'s own types, just the source of these two types shifts
- Depends only on `koi-common` (for `Res`, `Config`, `Pos` if kept here)

**Co-dependency note**

In the current monolith, `ast/file.rs` imports `ImportPath` and `ModulePath` from `module/`, while `module/path.rs` imports `ImportNode` from `ast/`. As separate crates this would be a circular dependency: `koi-ast → koi-sema → koi-ast`.

The resolution is to move `ImportPath` and `ModulePath` into `koi-ast`. Both types are derived directly from import token data (`From<&ImportNode> for ImportPath`) and from file paths — they are structural/syntactic by nature, not semantic. `koi-sema` then imports them from `koi-ast`, which is already a dependency, and the cycle is eliminated.

This also removes `koi-parser`'s only reason to depend on `koi-sema`: all three parser files (`parse.rs`, `passes.rs`, `depgraph.rs`) use only `ModulePath` and `ImportPath` from the sema layer, both of which now live in `koi-ast`.

---

### 3. `koi-scanner`

**Justification**

The scanner is a pure function: `Source` in, `Vec<Token>` out. It has no state that needs to persist after scanning, no dependency on modules or types, and is a natural first pipeline stage. As a standalone crate it can be tested, fuzzed, or benchmarked in complete isolation.

**Contents**
- `scanner/mod.rs` → `Scanner`, `scan()`
- `scanner/scanner_test.rs` → unit tests

**Public API**

```rust
pub fn scan(source: &Source, config: &Config) -> Res<Vec<Token>>;
```

**Changes needed**
- Move `src/scanner/` to `crates/koi-scanner/src/`
- Dependencies: `koi-ast` (for `Token`, `Source`), `koi-common` (for `Config`, `Res`, `Report`)
- No other changes needed; the scanner is already well-isolated

---

### 4. `koi-sema`

**Justification**

The type system, module system, interning context, and external import handling are four modules that are deeply intertwined in practice. `TypeId` is embedded in `Symbol`. `Module` is stored in `ModuleInterner` inside `Context`. Import headers serialize and deserialize `Module` data. Attempting to split these into separate crates would produce a tangle of cross-crate type references with almost no real isolation benefit — each "separate" crate would immediately re-depend on the others.

Grouping them as `koi-sema` reflects their shared purpose: they collectively represent the semantic model of a Koi program (what types exist, what modules exist, what symbols are in scope, what external libraries are available). This crate sits between the syntactic front end (`koi-ast`, `koi-parser`) and the analysis phase (`koi-typecheck`), and is the single shared dependency of both.

**Contents**
- `types/mod.rs` → `Type`, `TypeId`, `TypeKind`, `PrimitiveType`, `FunctionType`
- `types/ast.rs` → `TypedAst`, typed `Decl`/`Stmt`/`Expr` nodes
- `module/mod.rs` → `Module`, `ModuleId`, `ModuleKind`
- `module/symbols.rs` → `Symbol`, `SymbolId`, `SymbolKind`, `SymbolList`
- `module/namespace.rs` → `Namespace`, `NamespaceList`
- `context/mod.rs` → `Context`
- `context/types.rs` → `TypeInterner`
- `context/modules.rs` → `ModuleInterner`
- `context/symbols.rs` → `SymbolInterner`
- `imports/mod.rs`
- `imports/header.rs` → `create_header_file()`, `read_header_file()`
- `imports/libraries.rs` → `LibrarySet`

Note: `module/path.rs` (`ImportPath`, `ModulePath`) is **not** part of this crate — it moves to `koi-ast`. See the co-dependency note in the `koi-ast` section.

**Public API**

```rust
// Type system
pub type TypeId = usize;
pub enum TypeKind {
    Primitive(PrimitiveType),
    Pointer(TypeId),
    Array(TypeId),
    Function(FunctionType),
    ...
}
pub struct TypedAst { ... }

// Module system (ImportPath/ModulePath re-exported from koi-ast)
pub use koi_ast::{ImportPath, ModulePath};
pub struct Module { pub id: ModuleId, pub modpath: ModulePath, pub kind: ModuleKind, ... }
pub struct Symbol { pub name: String, pub kind: SymbolKind, pub exported: bool, ... }
pub struct SymbolList { ... }
pub struct Namespace { ... }

// Global context
pub struct Context {
    pub types: TypeInterner,
    pub modules: ModuleInterner,
    pub symbols: SymbolInterner,
    pub config: Config,
}

impl Context {
    pub fn new(config: Config) -> Self;
    pub fn intern_type(&mut self, kind: TypeKind) -> TypeId;
    pub fn intern_module(&mut self, module: Module) -> ModuleId;
    ...
}

// External imports
pub struct LibrarySet { pub headers: Vec<PathBuf>, pub archives: Vec<PathBuf> }
pub fn create_header_file(module: &Module, path: &Path) -> Res<()>;
pub fn read_header_file(path: &Path) -> Res<Module>;
```

**Changes needed**
- Move `src/types/`, `src/module/`, `src/context/`, and `src/imports/` to `crates/koi-sema/src/`, organized as submodules (`sema::types`, `sema::module`, `sema::context`, `sema::imports`)
- Remove `module/path.rs` from this crate — `ImportPath` and `ModulePath` now come from `koi-ast`; update all `use crate::module::{ImportPath, ModulePath}` references to `use koi_ast::{ImportPath, ModulePath}`
- Dependencies: `koi-ast`, `koi-common`
- `TypedAst` references AST node types — these come from `koi-ast`, which is already a dependency
- Ensure the header file serialization format is versioned so `.koi.h` files remain compatible across compiler versions (pre-existing concern worth addressing here)

---

### 5. `koi-parser`

**Justification**

The parser takes tokens and produces an AST. It is a classic pipeline stage that is completely independent of type checking or code generation. Separating it enables independent testing of the parser, and tools like IDEs or formatters could depend on `koi-parser` without pulling in the full compiler.

**Contents**
- `parser/mod.rs`
- `parser/parse.rs` → `Parser`, `parse_source_map()`
- `parser/passes.rs` → `validate_imports()`
- `parser/depgraph.rs` → `sort_by_dependency_graph()`
- `parser/tests/`

**Public API**

```rust
pub fn parse_source_map(modpath: &ModulePath, map: &SourceMap, config: &Config) -> Res<FileSet>;
pub fn sort_by_dependency_graph(filesets: Vec<FileSet>) -> Res<Vec<FileSet>>;
pub fn validate_imports(fileset: &FileSet, config: &Config) -> Res<()>;
```

**Changes needed**
- Move `src/parser/` to `crates/koi-parser/src/`
- Dependencies: `koi-ast` (for `ModulePath`, `ImportPath`, AST types), `koi-scanner`, `koi-common`
- No dependency on `koi-sema` — `ModulePath` and `ImportPath` now live in `koi-ast`, which is the only sema-layer type the parser ever used
- `depgraph.rs` uses `petgraph` — keep that dependency scoped to this crate
- The scanner is currently invoked from within the parser flow; that coupling is fine since `koi-scanner` is a dependency here

---

### 6. `koi-ir`

**Justification**

The IR is the compiler's internal representation after type checking, before code generation. It is consumed only by the lowering phase (write) and the two backends (read). Extracting it decouples the backends from each other and from the front end — if you add a third backend (LLVM, WASM), you only depend on `koi-ir`, not on the parser or type checker.

**Contents**
- `ir/mod.rs`
- `ir/nodes.rs` → `ProgramIR`, `Unit`, `Decl`, `FuncDecl`, `Ins`, `RValue`
- `ir/types.rs` → `IRType`, `IRTypeInterner`, `Primitive`
- `ir/sym.rs` → `SymTracker`
- `ir/print.rs` → IR pretty-printer

**Public API**

```rust
pub struct ProgramIR { pub units: Vec<Unit> }
pub struct Unit { pub types: IRTypeInterner, pub decls: Vec<Decl>, pub data: Vec<DataEntry> }
pub enum Ins { Store { .. }, Assign { .. }, Call { .. }, Return { .. }, If { .. }, ... }
pub enum RValue { Const(Literal), Param(usize), Temp(usize) }
pub enum IRType { Primitive(Primitive), Pointer(Box<IRType>), Function { .. } }
```

**Changes needed**
- Move `src/ir/` to `crates/koi-ir/src/`
- Dependencies: `koi-common` only (IR types are self-contained; primitive values may reference `Literal` from `koi-ast`, evaluate whether to duplicate or share)
- If `Literal` is shared, add `koi-ast` as a dependency; otherwise redefine a local `Value` enum to keep `koi-ir` fully independent — the latter is cleaner for a backend-facing interface

---

### 7. `koi-typecheck`

**Justification**

Type checking is the most complex phase. It reads a `FileSet`, resolves symbols through the `Context`, and emits a `TypedAst`. It is entirely independent of code generation and can be compiled and tested without any backend. It is also the most likely phase to change as the language evolves, so incremental builds benefit most from isolating it.

**Contents**
- `typecheck/mod.rs` → `check_filesets()`, `check_fileset()`
- `typecheck/module_check.rs` → `ModuleChecker` (3-pass: imports, globals, files)
- `typecheck/file_check.rs` → `FileChecker`
- `typecheck/tests/`

**Public API**

```rust
pub fn check_filesets(ctx: &mut Context, filesets: Vec<FileSet>) -> Res<Vec<TypedModule>>;
pub fn check_fileset(ctx: &mut Context, fileset: FileSet) -> Res<TypedModule>;
```

**Changes needed**
- Move `src/typecheck/` to `crates/koi-typecheck/src/`
- Dependencies: `koi-ast`, `koi-sema`, `koi-common`
- The return type `TypedModule` should be defined in `koi-sema` (alongside `TypedAst`) so it can be shared with the lowering phase without `koi-typecheck` and `koi-lower` depending on each other

---

### 8. `koi-lower`

**Justification**

Lowering translates the typed AST to IR. It is a clean transformation with clearly defined inputs (`TypedAst`, `Context`) and outputs (`Unit`). Once lowering is done, the front-end data structures are no longer needed. Separating this phase keeps the backend crates from ever touching AST types.

**Contents**
- `lower/mod.rs` → `emit_ir()`
- `lower/emit.rs` → `ModuleEmitter`, `FileEmitter`, `FunctionEmitter`

**Public API**

```rust
pub fn emit_ir(ctx: &Context, module_id: ModuleId) -> Res<Unit>;
```

**Changes needed**
- Move `src/lower/` to `crates/koi-lower/src/`
- Dependencies: `koi-ir`, `koi-sema`, `koi-common`
- `VarTable` from `util/vartable.rs` is used during lowering; move it into `koi-lower` since it is not used elsewhere, or keep it in `koi-common` — moving it is cleaner

---

### 9. `koi-codegen-c`

**Justification**

The C backend is a complete, self-contained code generation strategy. It takes IR and produces C source files, then invokes gcc. By separating it, you can add or remove backends without touching any other crate. It is also the most likely place where compiler users might want to substitute their own backend.

**Contents**
- `build/c.rs` (top-level C build entry)
- `build/c/ast.rs` → C AST types
- `build/c/emit.rs` → `CEmitter`
- `build/c/tests.rs`

**Public API**

```rust
pub fn build(ir: &ProgramIR, config: &Config, imports: &LibrarySet, out_dir: &Path) -> Res<PathBuf>;
```

**Changes needed**
- Move `src/build/c*` to `crates/koi-codegen-c/src/`
- Dependencies: `koi-ir`, `koi-sema` (for `LibrarySet`), `koi-common`
- No logic changes; already well-isolated from the front end

---

### 10. `koi-codegen-x86`

**Justification**

Same reasoning as `koi-codegen-c`. The x86-64 backend is independent and self-contained. Separating it means adding LLVM or WASM backends later has zero impact on the C backend or any other crate.

**Contents**
- `build/x86.rs` (top-level x86 build entry)
- `build/x86/assembly.rs` → x86 AST, instruction types
- `build/x86/assemble.rs` → `AssemblyEmitter`
- `build/x86/tests.rs`

**Public API**

```rust
pub fn build(ir: &ProgramIR, config: &Config, imports: &LibrarySet, out_dir: &Path) -> Res<PathBuf>;
```

**Changes needed**
- Move `src/build/x86*` to `crates/koi-codegen-x86/src/`
- Dependencies: `koi-ir`, `koi-sema` (for `LibrarySet`), `koi-common`
- The `gcc_available()` check currently lives in `build/mod.rs` — move it into `koi-common` or duplicate it in each backend crate (duplication is fine for a one-liner)

---

### 11. `koi-driver`

**Justification**

The driver is the orchestrator that calls all phases in order. It lives at the top of the dependency graph. As a library crate, it can be used by the CLI binary, by tests that want to run the full pipeline, and potentially by language server implementations. Keeping it separate from `main.rs` enables this reuse.

**Contents**
- `driver.rs` → `compile()`, pipeline orchestration

**Public API**

```rust
pub fn compile(project: &Project, options: &CompileOptions, config: &Config) -> Res<()>;
```

**Changes needed**
- Move `src/driver.rs` to `crates/koi-driver/src/lib.rs`
- Dependencies: all other crates (this is the integration point)
- Currently `driver.rs` also handles some source collection I/O — consider whether that belongs here or in `koi-common`'s I/O utilities

---

## Changes Not Worth Making

**Do not split:**

- `koi-ast` into sub-crates (tokens, nodes, source) — they are tightly coupled and tiny; splitting adds friction with no benefit
- The two backends into further sub-crates — each backend is already compact and internally consistent
- `koi-common` into `koi-error` + `koi-config` + `koi-util` — all three are tiny and always co-depended upon; the split adds boilerplate
- `koi-sema` any further — types, modules, context, and imports are already split into submodules within the crate; the crate boundary is where the meaningful isolation ends. Note that `module/path.rs` intentionally lives in `koi-ast` to avoid a circular dependency — this is not an exception to the rule but the rule applied correctly

---

## Migration Plan

The split can be done incrementally without breaking the compiler at any point:

**Phase 1 — Workspace setup**
1. Add a `[workspace]` table to the root `Cargo.toml` with `members = ["crates/*", "."]`
2. Create `crates/` directory
3. No code moves yet; verify the workspace builds

**Phase 2 — Bottom-up extraction (no-dependency crates first)**
4. Extract `koi-common` (error, config, util) — touches nothing else
5. Extract `koi-ast` — depends only on `koi-common`
6. Extract `koi-ir` — depends only on `koi-common`

**Phase 3 — Semantic layer**
7. Extract `koi-sema` (types, module, context, imports — but *not* `module/path.rs`) — depends on `koi-ast`, `koi-common`

**Phase 4 — Pipeline stages**
8. Extract `koi-scanner` — depends on `koi-ast`, `koi-common`
9. Extract `koi-parser` — depends on `koi-ast`, `koi-scanner`, `koi-common` (no `koi-sema` dependency)
10. Extract `koi-typecheck` — depends on `koi-ast`, `koi-sema`, `koi-common`
11. Extract `koi-lower` — depends on `koi-ir`, `koi-sema`, `koi-common`

**Phase 5 — Backends and driver**
12. Extract `koi-codegen-c` and `koi-codegen-x86`
13. Extract `koi-driver`
14. Slim `src/` down to `main.rs` + `cmd.rs` (the binary crate)

At each phase, run `cargo test` and `cargo build` before moving to the next. Because each extraction moves code unchanged, regressions will be caused by visibility issues (making things `pub` that were previously `pub(crate)`) rather than logic errors.

---

## Summary Table

| Crate | Phase | Key Dependency | Purpose |
|---|---|---|---|
| `koi-common` | Foundation | (none) | Errors, config, utilities |
| `koi-ast` | Frontend | `koi-common` | Tokens, AST nodes, source map, module paths |
| `koi-scanner` | Frontend | `koi-ast` | Tokenizer |
| `koi-parser` | Frontend | `koi-ast`, `koi-scanner` | Parser, dep graph |
| `koi-sema` | Semantic | `koi-ast`, `koi-common` | Types, modules, symbols, context, imports |
| `koi-ir` | IR | `koi-common` | Intermediate representation |
| `koi-typecheck` | Semantic | `koi-sema`, `koi-ast` | Type checker |
| `koi-lower` | Lower | `koi-ir`, `koi-sema` | AST → IR lowering |
| `koi-codegen-c` | Codegen | `koi-ir`, `koi-sema` | C backend |
| `koi-codegen-x86` | Codegen | `koi-ir`, `koi-sema` | x86-64 backend |
| `koi-driver` | Driver | all | Compilation orchestrator |
