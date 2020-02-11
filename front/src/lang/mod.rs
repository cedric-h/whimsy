use crate::{compiler, vm};
use vm::pyobject::ItemProtocol;

mod draw_commands;

pub struct Lang {
    vm: vm::VirtualMachine,
    scope: vm::scope::Scope,
    ast: vm::obj::objcode::PyCodeRef,
}

impl Lang {
    pub fn new() -> Self {
        log::info!("creating vm!");
        let vm = vm::VirtualMachine::new(vm::PySettings {
            initialization_parameter: vm::InitParameter::InitializeInternal,
            ..Default::default()
        });
        let scope = vm.new_scope_with_builtins();

        log::info!("compiling starter source!");
        let ast = vm
            .compile("", compiler::compile::Mode::Exec, "<start>".to_string())
            .map_err(|err| vm.new_syntax_error(&err))
            .unwrap_or_else(|e| {
                panic!("{}", Self::format_err(&vm, e));
            });

        Lang { vm, scope, ast }
    }

    pub fn format_err(
        vm: &vm::VirtualMachine,
        err: vm::pyobject::PyRef<vm::exceptions::PyBaseException>,
    ) -> String {
        let mut out = Vec::new();
        vm::exceptions::write_exception(&mut out, vm, &err).expect("could not format exception");
        String::from_utf8(out).unwrap()
    }

    /// Replaces the AST if the provided source code text compiles.
    /// If it doesn't compile, the state isn't changed at all.
    pub fn try_update_ast(&mut self, text: &str) -> Result<(), String> {
        let ast = self
            .vm
            .compile(
                text,
                compiler::compile::Mode::Exec,
                "<embedded>".to_string(),
            )
            .map_err(|err| self.vm.new_syntax_error(&err))
            .and_then(|code_obj| {
                self.vm.run_code_obj(code_obj.clone(), self.scope.clone())?;
                Ok(code_obj)
            })
            .map_err(|e| Self::format_err(&self.vm, e))?;
        self.ast = ast;

        Ok(())
    }

    #[inline]
    pub fn run_ast(&self) -> Result<(), String> {
        self.vm
            .run_code_obj(self.ast.clone(), self.scope.clone())
            .map_err(|e| Self::format_err(&self.vm, e))
            .map(|_| ())
    }

    #[inline]
    pub fn set_global(&mut self, var: &'static str, val: f64) {
        use vm::pyobject::IntoPyObject;

        self.scope
            .globals
            .set_item(var, val.into_pyobject(&self.vm).unwrap(), &self.vm)
            .unwrap();
    }
}
