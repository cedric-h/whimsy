mod draw_commands;

struct Lang {
    vm: vm::VirtualMachine,
    scope: vm::scope::Scope,
    ast: vm::obj::objcode::PyCodeRef,
}

impl Lang {
    fn new() -> Result<Self> {
        log::info!("creating vm!");
        let vm = vm::VirtualMachine::new(vm::PySettings {
            initialization_parameter: vm::InitParameter::InitializeInternal,
            ..Default::default()
        });
        let scope = vm.new_scope_with_builtins();

        log::info!("compiling starter source!");
        let ast = vm.compile(START_STRING, Exec, "<start>".to_string())
                .map_err(|err| vm.new_syntax_error(&err))
                .unwrap_or_else(|e| {
                    panic!("{}", Self::format_err(&vm, e));
                });

        Ok(Lang {
            vm,
            scope,
            ast,
        })
    }

    fn format_err(vm: &vm::VirtualMachine, err: vm::pyobject::PyRef<vm::exceptions::PyBaseException>) -> String {
        let mut out = Vec::new();
        vm::exceptions::write_exception(&mut out, vm, &err).expect("could not format exception");
        String::from_utf8(out).unwrap()
    }

    fn new_ast(&mut self, text: &str) -> Result<String, ()> {
        let ast = self.vm
            .compile(text, Exec, "<embedded>".to_string())
            .map_err(|err| self.vm.new_syntax_error(&err))
            .and_then(|code_obj| {
                self.vm.run_code_obj(code_obj.clone(), self.scope.clone())?;
                Ok(code_obj)
            })
            .map_err(|e| Self::format_err(&self.vm, e))?;

        self.to_editor.dispatch_event(&Self::err_event(String::new()));
        self.ast = ast;

        Ok(())
    }

    #[inline]
    fn run_ast(&self) -> Result<String, ()> {
        self
            .vm
            .run_code_obj(self.ast.clone(), self.scope.clone())
            .map_err(|e| Self::format_err(&self.vm, e))?;
    }

    #[inline]
    fn set_global(&mut self, var: &'static str, val: f64) {
        use vm::pyobject::IntoPyObject;

        self
            .scope
            .globals
            .set_item("time", time.into_pyobject(&self.vm).unwrap(), &self.vm)
            .unwrap();
    }
}
