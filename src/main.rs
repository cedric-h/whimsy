use rustpython_compiler as compiler;
use rustpython_vm as vm;
use std::sync::mpsc;
use vm::pyobject::ItemProtocol;

pub mod draw {
    use quicksilver::geom::{Rectangle, Transform};
    use quicksilver::graphics::Color;

    #[derive(Debug, Clone, PartialEq)]
    pub enum DrawCommand {
        Rect(Rectangle),
        Color(Color),
        Transform(Transform),
        SaveTransform,
        RestoreTransform,
    }
}

fn main() -> vm::pyobject::PyResult<()> {
    let mut input = String::new();
    let stdin = std::io::stdin();

    let vm = vm::VirtualMachine::new(vm::PySettings::default());
    let scope = vm.new_scope_with_builtins();
    let draw_buffer = add_render_methods(&vm, &scope)?;

    loop {
        input.clear();
        stdin
            .read_line(&mut input)
            .expect("Failed to read line of input");

        // this line also automatically prints the output
        // (note that this is only the case when compile::Mode::Single is passed to vm.compile)
        match vm
            .compile(
                &input,
                compiler::compile::Mode::Single,
                "<embedded>".to_string(),
            )
            .map_err(|err| vm.new_syntax_error(&err))
            .and_then(|code_obj| vm.run_code_obj(code_obj, scope.clone()))
        {
            Ok(output) => {
                // store the last value in the "last" variable
                if !vm.is_none(&output) {
                    scope.globals.set_item("last", output, &vm)?;
                }
            }
            Err(e) => {
                vm::exceptions::print_exception(&vm, &e);
            }
        }

        while let Ok(draw_cmd) = draw_buffer.try_recv() {
            println!("{:?}", draw_cmd);
        }
    }

    Ok(())
}

fn add_render_methods(
    vm: &vm::VirtualMachine,
    scope: &vm::scope::Scope,
) -> vm::pyobject::PyResult<mpsc::Receiver<draw::DrawCommand>> {
    use draw::DrawCommand::*;
    use num_traits::ToPrimitive;
    use vm::function::PyFuncArgs;
    use vm::obj::objfloat::PyFloat;
    use vm::obj::objint::PyInt;
    use vm::pyobject::{PyObject, PyObjectPayload, PyResult};
    use vm::VirtualMachine;

    type ArgsList = Vec<std::rc::Rc<PyObject<dyn PyObjectPayload + 'static>>>;

    let (draws_tx, draws_rx) = mpsc::channel();

    macro_rules! expose_fn {
        ( $name:literal, $tx:ident, $fn:expr $(,)? ) => {
            scope.globals.set_item(
                $name,
                vm.context().new_function({
                    let $tx = draws_tx.clone();
                    $fn
                }),
                &vm,
            )
        };
    }

    expose_fn!("rect", d, move |vm: &VirtualMachine, args: PyFuncArgs| {
        use quicksilver::geom::{Rectangle, Transform, Vector};
        use std::rc::Rc;

        let raw = args
            .args
            .into_iter()
            .take(4)
            .map(|x| {
                x.downcast::<PyFloat>()
                    .map(|f| f.to_f64() as f32)
                    .or_else(|e| {
                        e.downcast::<PyInt>()
                            .map(|i| i.as_bigint().clone().to_f32().unwrap())
                    })
                    .map_err(|e| {
                        vm.new_value_error(format!(
                            "`rect` passed {}, expected number",
                            e.to_string()
                        ))
                    })
            })
            .collect::<PyResult<Vec<f32>>>()?;

        if raw.len() != 4 {
            Err(vm.new_index_error(format!("`rect` passed {} args, needed 4", raw.len())))
        } else {
            let mut vectored = raw
                .chunks_exact(2)
                .map(|fs| Vector::new(fs[0], fs[1]))
                .collect::<Vec<_>>();

            let size = vectored.pop().unwrap();
            let pos = vectored.pop().unwrap();

            d.send(SaveTransform).unwrap();
            d.send(Transform(Transform::translate(pos + size / 2.0)))
                .unwrap();
            d.send(Rect(Rectangle::new(-size / 2.0, size))).unwrap();
            d.send(RestoreTransform).unwrap();

            Ok(vm.get_none())
        }
    })?;
    expose_fn!("push", d, move || d
        .send(draw::DrawCommand::SaveTransform)
        .unwrap())?;
    expose_fn!("pop", d, move || d
        .send(draw::DrawCommand::RestoreTransform)
        .unwrap())?;

    return Ok(draws_rx);
}
