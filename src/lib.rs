// RustPython
use rustpython_compiler as compiler;
use rustpython_vm as vm;
use vm::pyobject::ItemProtocol;
use compiler::compile::Mode::Exec;
// STD
use std::sync::mpsc;
// WASM
use wasm_bindgen::prelude::*;
// quicksilver
use quicksilver::{
    graphics::Graphics,
    lifecycle::{run, EventStream, Settings, Window},
    Result,
};

const START_STRING: &'static str = r#"
move(300, 300)
zoom(100)
for i in range(10):
    spin(time)
    move(1, 0)

    push()
    zoom(0.8 * (i/10) + 0.2)
    fill(1, 1, 1, i/10)
    rect(-.5, -.5, 1, 1)
    pop()

"#;

pub mod draw {
    use quicksilver::geom::{Rectangle, Transform};
    use quicksilver::graphics::Color;

    #[derive(Debug, Clone, PartialEq)]
    pub enum DrawCommand {
        Rect(Rectangle),
        Fill(Color),
        Transform(Transform),
        PopTransform,
        PushTransform,
    }
}

struct QuickTest {
    // text editing
    text: String,
    error: Option<String>,

    // rendering
    timer: usize,
    draw_cmds: mpsc::Receiver<draw::DrawCommand>,

    // lang
    vm: vm::VirtualMachine,
    scope: vm::scope::Scope,
    ast: vm::obj::objcode::PyCodeRef,
    ast_dirty: bool,
}

impl QuickTest {
    fn new() -> Result<QuickTest> {
        log::info!("creating quicktest!");
        let vm = vm::VirtualMachine::new(vm::PySettings {
            initialization_parameter: vm::InitParameter::InitializeInternal,
            ..Default::default()
        });
        let scope = vm.new_scope_with_builtins();

        log::info!("registering drawing commands!");
        let draw_cmds = add_render_methods(&vm, &scope).unwrap();

        log::info!("compiling starter source!");
        let ast = vm.compile(START_STRING, Exec, "<start>".to_string())
                .map_err(|err| vm.new_syntax_error(&err))
                .unwrap_or_else(|e| {
                    panic!("{}", Self::format_err(&vm, e).unwrap());
                });

        Ok(QuickTest {
            // text editing
            text: START_STRING.to_string(),
            error: None,

            // rendering
            draw_cmds,
            timer: 0,

            // ast
            vm,
            scope,
            ast_dirty: false,
            ast,
        })
    }

    fn format_err(vm: &vm::VirtualMachine, err: vm::pyobject::PyRef<vm::exceptions::PyBaseException>) -> Option<String> {
        let mut out = Vec::new();
        vm::exceptions::write_exception(&mut out, vm, &err);
        Some(String::from_utf8(out).unwrap())
    }

    fn update(&mut self) -> Result<()> {
        use vm::pyobject::IntoPyObject;

        self.timer += 1;
        self.scope.globals.set_item("time", (self.timer as f64).into_pyobject(&self.vm).unwrap(), &self.vm).unwrap();

        if self.ast_dirty {
            match self.vm
                .compile(&self.text, Exec, "<embedded>".to_string())
                .map_err(|err| self.vm.new_syntax_error(&err))
                .and_then(|code_obj| {
                    self.vm.run_code_obj(code_obj.clone(), self.scope.clone())?;
                    Ok(code_obj)
                })
            {
                Ok(ast) => {
                    self.error = None;
                    self.ast = ast;
                }
                Err(err) => self.error = Self::format_err(&self.vm, err),
            };
            self.ast_dirty = false;
        }

        Ok(())
    }

    fn draw(&mut self, gfx: &mut Graphics) -> Result<()> {
        use quicksilver::graphics::Color;

        // Remove any lingering artifacts from the previous frame
        gfx.clear(Color::BLACK);

        let mut color = Color::WHITE;
        let mut transforms = vec![quicksilver::geom::Transform::IDENTITY];

        if let Err(err) = self.vm.run_code_obj(self.ast.clone(), self.scope.clone()) {
            // this really shouldn't happen, we should save asts that don't error.
            log::error!("bad ast slipped in!");
            self.error = Self::format_err(&self.vm, err);
        }
        while let Ok(cmd) = self.draw_cmds.try_recv() {
            use draw::DrawCommand::*;
            match cmd {
                Rect(rect) => gfx.fill_rect(&rect, color),
                Fill(new_color) => color = new_color,
                Transform(t) => {
                    let now = transforms.last_mut().unwrap();
                    *now = *now * t;
                    gfx.set_transform(*now);
                }
                PushTransform => {
                    let now = transforms.last().unwrap().clone();
                    transforms.push(now);
                    gfx.set_transform(now);
                }
                PopTransform => {
                    if transforms.len() >= 2 {
                        transforms.pop();
                    }
                    gfx.set_transform(*transforms.last().unwrap());
                }
            }
        }

        Ok(())
    }
}

async fn app(win: Window, mut gfx: Graphics, mut events: EventStream, new_code_events: wasm_bindgen::JsValue) -> Result<()> {
    use wasm_bindgen::JsCast;

    // create callback
    let (new_code_tx, new_code_rx) = mpsc::channel();
    let handler = Closure::wrap(Box::new(move |e: web_sys::CustomEvent| {
        let code = e.detail().as_string().expect("The detail field of the 'code' event must be a string");
        new_code_tx
            .send(code)
            .expect("Couldn't send new code");
    }) as Box<dyn FnMut(web_sys::CustomEvent)>);
    new_code_events
        .dyn_into::<web_sys::EventTarget>()
        .expect("third variable passed to main not an EventTarget")
        .add_event_listener_with_callback("code", handler.as_ref().unchecked_ref())
        .expect("can't listen to 'code' events on EventTarget passed");
    // forget the handler to keep it alive
    handler.forget();

    let mut qt = QuickTest::new()?;
    loop {
        while let Some(_) = events.next_event().await { }

        while let Ok(new_code) = new_code_rx.try_recv() {
            log::info!("{} recieved, setting!", new_code);
            qt.text = new_code;
            qt.ast_dirty = true;
        }
        qt.update()?;
        qt.draw(&mut gfx)?;
        gfx.present(&win)?;
    }
}

#[wasm_bindgen]
pub fn main(x: f32, y: f32, new_code_events: wasm_bindgen::JsValue) {
    run(
        Settings {
            size: quicksilver::geom::Vector::new(x, y).into(),
            title: "QuickTest",
            multisampling: Some(16),
            ..Settings::default()
        },
        move |w, g, e| app(w, g, e, new_code_events)
    );
}

fn add_render_methods(
    vm: &vm::VirtualMachine,
    scope: &vm::scope::Scope,
) -> vm::pyobject::PyResult<mpsc::Receiver<draw::DrawCommand>> {
    use quicksilver::geom::Transform as Matrix;
    use draw::DrawCommand::*;
    use quicksilver::graphics::Color;
    use vm::function::OptionalArg;
    use vm::obj::objfloat::IntoPyFloat as Num;
    use vm::obj::objstr::PyStringRef;
    use vm::pyobject::Either;

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

    // RENDERING
    expose_fn!("rect", d, move |px: Num, py: Num, sx: Num, sy: Num| {
        use quicksilver::geom::{ Rectangle, Vector };

        let pos = Vector::new(px.to_f64() as f32, py.to_f64() as f32);
        let size = Vector::new(sx.to_f64() as f32, sy.to_f64() as f32);

        d.send(PushTransform).unwrap();
        d.send(Transform(Matrix::translate(pos + size / 2.0)))
            .unwrap();
        d.send(Rect(Rectangle::new(-size / 2.0, size))).unwrap();
        d.send(PopTransform).unwrap();

        Ok(())
    })?;
    const COLORS: &'static [(&'static str, Color)] = &[
        ("white", Color::WHITE),
        ("black", Color::BLACK),
        ("red", Color::RED),
        ("orange", Color::ORANGE),
        ("yellow", Color::YELLOW),
        ("green", Color::GREEN),
        ("cyan", Color::CYAN),
        ("blue", Color::BLUE),
        ("magenta", Color::MAGENTA),
        ("purple", Color::PURPLE),
        ("indigo", Color::INDIGO),
    ];
    let colors: fxhash::FxHashMap<String, Color> = COLORS
        .iter()
        .copied()
        .map(|(s, c)| (s.to_string(), c))
        .collect();
    expose_fn!(
        "fill",
        d,
        move |first: Either<PyStringRef, Num>,
              g: OptionalArg<Num>,
              b: OptionalArg<Num>,
              a: OptionalArg<Num>,
              vm: &vm::VirtualMachine| {
            d.send(Fill(match first {
                Either::A(color_name) => colors
                    .get(color_name.as_str())
                    .copied()
                    .ok_or_else(|| vm.new_lookup_error(format!("Unknown color `{}`", color_name))),
                Either::B(r) => {
                    let fields = vec![Some(r), g.into_option(), b.into_option(), a.into_option()]
                        .into_iter()
                        .filter_map(|f| f.map(|x| x.to_f64() as f32))
                        .collect::<Vec<f32>>();
                    if fields.len() != 4 {
                        Err(vm.new_value_error(format!(
                            concat!(
                                "`fill` takes either a color name or 4 number parameters ",
                                "representing red, green, blue, and alpha (transparency). ",
                                "However, only {} parameters were provided."
                            ),
                            fields.len()
                        )))
                    } else {
                        Ok(quicksilver::graphics::Color {
                            r: fields[0],
                            g: fields[1],
                            b: fields[2],
                            a: fields[3],
                        })
                    }
                }
            }?))
            .unwrap();
            Ok(vm.get_none())
        }
    )?;

    // TRANSFORMS
    expose_fn!("move", d, move |x: Num, y: Num| {
        d.send(Transform(Matrix::translate((x.to_f64() as f32, y.to_f64() as f32)))).unwrap();
        Ok(())
    })?;
    expose_fn!("spin", d, move |angle: Num| {
        d.send(Transform(Matrix::rotate(angle.to_f64() as f32))).unwrap();
        Ok(())
    })?;
    expose_fn!("zoom", d, move |x: Num, y: OptionalArg<Num>| {
        let x = x.to_f64() as f32;
        let y = y.into_option().map(|x| x.to_f64() as f32);
        d.send(Transform(Matrix::scale((x, y.unwrap_or(x))))).unwrap();
        Ok(())
    })?;
    expose_fn!("push", d, move || d
        .send(draw::DrawCommand::PushTransform)
        .unwrap())?;
    expose_fn!("pop", d, move || d
        .send(draw::DrawCommand::PopTransform)
        .unwrap())?;

    return Ok(draws_rx);
}
