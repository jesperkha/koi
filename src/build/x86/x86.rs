use std::{
    collections::HashMap,
    process::{Command, Stdio},
};

use tracing::info;

use crate::{
    build::x86::reg_alloc::RegAllocator,
    config::{Config, PathManager},
    imports::LibrarySet,
    ir::{AssignIns, ConstId, IRType, IRVisitor, Ir, LValue, StoreIns, Unit, Value},
    util::{FilePath, cmd, write_file},
};

pub enum LinkMode {
    /// Link as executable ELF file
    Executable,
    /// Link to static library file (.a)
    Library,
}

pub struct BuildConfig {
    pub linkmode: LinkMode,
    /// Where to output temp files (.s .o)
    pub tmpdir: String,
    /// Filepath out output executable/object file
    pub target_name: String,
    /// Directory to output target file(s)
    pub outdir: String,
}

// TODO: high level assembly tree with string generation

/// Build and compile an x86-64 executable or shared object file.
pub fn build(
    ir: Ir,
    buildcfg: BuildConfig,
    config: &Config,
    pm: &PathManager,
    libset: &LibrarySet,
) -> Result<(), String> {
    info!("Building for x86-64. Output: {}", buildcfg.target_name);

    if !gcc_available() {
        return Err("Failed to run gcc. Make sure it's installed and in PATH.".into());
    }

    let mut asm_files = Vec::new();

    for unit in ir.units {
        info!("Assembling module {}", unit.modpath.path());
        let filepath = format!("{}/{}.s", buildcfg.tmpdir, unit.modpath.to_underscore());
        let source = X86Builder::new(config).build(unit)?;

        info!("Writing file {}", filepath);
        write_file(&filepath.as_str().into(), &source)?;
        asm_files.push(filepath);
    }

    let mut linker_flags = vec![];
    for lib in libset.archives() {
        linker_flags.push(format!("{}", lib));
    }

    match buildcfg.linkmode {
        LinkMode::Executable => {
            info!("Compiling executable");

            let mut args = asm_files;
            args.push("-nostartfiles".into());

            let entry_file = pm.library_path().join("entry.s");
            args.push(entry_file.to_string());
            let target_path = FilePath::from(&buildcfg.outdir).join(&buildcfg.target_name);
            args.push(format!("-o{}", target_path));
            args.extend_from_slice(&linker_flags);
            cmd("gcc", &args)?;
        }
        LinkMode::Library => {
            info!("Compiling static library");

            let mut objfiles = Vec::new();
            for asmfile in &asm_files {
                let objfile = asmfile.replace(".s", ".o");
                cmd(
                    "gcc",
                    &[
                        "-nostartfiles".into(),
                        "-c".into(),
                        asmfile.into(),
                        format!("-o{}", objfile),
                    ],
                )?;
                objfiles.push(objfile);
            }
            let target_path =
                FilePath::from(&buildcfg.outdir).join(&format!("lib{}.a", buildcfg.target_name));

            let mut args = vec!["rcs".into(), target_path.to_string()];
            args.extend_from_slice(&objfiles);
            args.extend_from_slice(&linker_flags);
            cmd("ar", &args)?;
        }
    }

    Ok(())
}

fn gcc_available() -> bool {
    Command::new("gcc")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

struct X86Builder<'a> {
    _config: &'a Config,
    regmap: HashMap<ConstId, String>,
    parammap: HashMap<ConstId, String>,
    alloc: RegAllocator,
    stacksize: usize,

    head: Writer,
    text: Writer,
    data: Writer,
}

struct Writer {
    indent: usize,
    content: String,
}

enum LVal {
    Reg(String),
    Stack(String),
}

#[derive(Clone)]
enum RVal {
    Imm(String),
    Reg(String),
    Data(String),
    Stack(String),
}

impl Writer {
    fn new() -> Self {
        Self {
            indent: 0,
            content: String::new(),
        }
    }

    fn write(&mut self, s: &str) {
        self.content
            .push_str(&format!("{}{}", "    ".repeat(self.indent), s));
    }

    fn writeln(&mut self, s: &str) {
        self.write(&format!("{}\n", s));
    }

    fn push(&mut self) {
        self.indent += 1
    }

    fn pop(&mut self) {
        self.indent -= 1;
    }

    fn append(&mut self, other: &Writer) {
        self.content.push_str(&other.content);
        self.content.push_str("\n");
    }
}

impl<'a> X86Builder<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            _config: config,

            head: Writer::new(),
            text: Writer::new(),
            data: Writer::new(),

            regmap: HashMap::new(),
            parammap: HashMap::new(),
            alloc: RegAllocator::new(),
            stacksize: 0,
        }
    }

    pub fn build(mut self, unit: Unit) -> Result<String, String> {
        self.head.writeln(".intel_syntax noprefix");
        self.data.push();

        for ins in &unit.ins {
            ins.accept(&mut self);
        }

        let mut src = Writer::new();
        src.append(&self.head);

        src.writeln(".section .data\n");
        src.append(&self.data);

        src.writeln(".section .text\n");
        src.append(&self.text);

        src.writeln(".section .note.GNU-stack,\"\",@progbits\n");

        Ok(src.content)
    }

    fn push(&mut self) {
        self.text.push();
    }

    fn pop(&mut self) {
        self.text.pop();
    }

    fn bind(&mut self, id: ConstId, reg: &str) {
        self.regmap.insert(id, reg.to_string());
    }

    fn get(&self, id: ConstId) -> &str {
        self.regmap.get(&id).expect("unknown const id")
    }

    fn bind_param(&mut self, id: ConstId, location: &str) {
        self.parammap.insert(id, location.to_string());
    }

    fn get_param(&self, id: ConstId) -> &str {
        self.parammap.get(&id).expect("unknown const id")
    }

    /// Convert IR Value to RVal
    fn rval(&self, v: &Value) -> RVal {
        match v {
            Value::Void => panic!("cannot get value of void type"),
            Value::Int(n) => RVal::Imm(n.to_string()),
            Value::Const(id) => RVal::Stack(self.get(*id).to_string()),
            Value::Param(id) => RVal::Stack(self.get_param(*id).to_string()),
            Value::Float(_) => todo!(),
            Value::Function(_) => todo!(),
            Value::Data(name) => RVal::Data(format!("[rip + .{}]", name)),
        }
    }

    /// Convert IR Value to LVal
    fn lval(&self, v: &LValue) -> LVal {
        match v {
            LValue::Const(id) => LVal::Stack(self.get(*id).to_string()),
            LValue::Param(id) => LVal::Stack(self.get_param(*id).to_string()),
        }
    }

    /// Checks L and R val to use correct mov instruction (mov, lea).
    /// Prints intermeditate steps if necessary (eg, dest and value are stack).
    fn mov(&mut self, dest: LVal, value: RVal, ty: &IRType) {
        let fmt = match dest {
            LVal::Reg(reg) => match &value {
                RVal::Imm(s) | RVal::Reg(s) | RVal::Stack(s) => format!("mov {}, {}", reg, s),
                RVal::Data(s) => format!("lea {}, {}", reg, s),
            },
            LVal::Stack(dest) => match &value {
                RVal::Imm(s) | RVal::Reg(s) => format!("mov {}, {}", dest, s),
                RVal::Data(_) | RVal::Stack(_) => {
                    self.mov(LVal::Reg("rax".to_string()), value.clone(), &ty);
                    format!("mov {}, rax", dest)
                }
            },
        };

        self.text.writeln(&fmt);
    }

    /// Increases stack size and returns location for requested size.
    fn stack_alloc(&mut self, size: usize) -> String {
        self.stacksize += size.max(4);
        let directive = match size {
            1 => "BYTE",
            2 => "WORD",
            4 => "DWORD",
            8 => "QWORD",
            _ => panic!("illegal size: {}", size),
        };
        format!("{} PTR [rbp-{}]", directive, self.stacksize)
    }
}

impl<'a> IRVisitor<()> for X86Builder<'a> {
    fn visit_func(&mut self, f: &crate::ir::FuncInst) {
        // Function label
        if f.public {
            self.text.writeln(&format!(".globl {}", f.name));
        }
        self.text.writeln(&format!("{}:", f.name));

        // Push new stack frame
        self.push();
        self.stacksize = 0;
        self.text.writeln("push rbp");
        self.text.writeln("mov rbp, rsp");

        if f.stacksize > 0 {
            let rounded_size = round_up_to_mult_of_16(f.stacksize);
            self.text.writeln(&format!("sub rsp, {}", rounded_size));
        }

        // Put params on stack
        self.alloc.reset_params();
        for (i, ty) in f.params.iter().enumerate() {
            let dest = self.stack_alloc(ty.size());
            let reg = self.alloc.next_param_reg(ty);

            self.bind_param(i, &dest);
            self.mov(LVal::Stack(dest), RVal::Reg(reg), ty);
        }

        for ins in &f.body {
            ins.accept(self);
        }

        self.pop();
    }

    fn visit_ret(&mut self, ty: &crate::ir::IRType, v: &crate::ir::Value) {
        // If not void
        if ty.size() != 0 {
            self.mov(LVal::Reg(self.alloc.return_reg(ty)), self.rval(v), ty);
        }
        self.text.writeln("leave");
        self.text.writeln("ret\n");
    }

    fn visit_store(&mut self, ins: &StoreIns) {
        let loc = self.stack_alloc(ins.ty.size());
        self.bind(ins.id, &loc);
        self.mov(LVal::Stack(loc), self.rval(&ins.value), &ins.ty);
    }

    fn visit_assign(&mut self, ins: &AssignIns) -> () {
        self.mov(self.lval(&ins.lval), self.rval(&ins.value), &ins.ty);
    }

    fn visit_call(&mut self, c: &crate::ir::CallIns) -> () {
        self.alloc.reset_params();
        match &c.callee {
            Value::Function(name) => {
                for arg in &c.args {
                    let dest = self.alloc.next_param_reg(&arg.0);
                    self.mov(LVal::Reg(dest), self.rval(&arg.1), &arg.0);
                }
                self.text.writeln(&format!("call {}", name));
                self.bind(c.result, &self.alloc.return_reg(&c.ty));
            }
            _ => panic!("invalid call callee"),
        }
    }

    fn visit_static_string(&mut self, d: &crate::ir::StringDataIns) -> () {
        //self.data.writeln(&format!(".local .{}", d.name));
        self.data
            .writeln(&format!(".{}: .asciz \"{}\"", d.name, d.value));
    }

    fn visit_extern(&mut self, f: &crate::ir::ExternFuncInst) -> () {
        self.head.writeln(&format!(".extern {}", f.name));
    }
}

fn round_up_to_mult_of_16(n: usize) -> usize {
    (n + 15) & !15
}
