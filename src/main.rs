use rustpython_compiler as compiler;
use rustpython_vm as vm;
use std::sync::mpsc;
use vm::pyobject::ItemProtocol;
use compiler::compile::Mode::Exec;
use quicksilver::{
    geom::Vector,
    graphics::{
        Background::{Col, Img},
        Color, Font, FontStyle,
    },
    lifecycle::{run, Asset, Event, Settings, State, Window},
    Result,
};

const START_STRING: &'static str = r#"
move(450, 250)
rect(100, 100, 100, 100)

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
    font: Asset<Font>,
    text: String,
    cursor: usize,
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
    fn cursor_left(&self) -> usize {
        self.cursor.checked_sub(1).unwrap_or(self.text.len() - 1)
    }
    fn cursor_right(&self) -> usize {
        if self.cursor + 1 < self.text.len() - 1 {
            self.cursor + 1
        } else {
            0
        }
    }
    fn type_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor = self.cursor_right();
    }
}

impl State for QuickTest {
    fn new() -> Result<QuickTest> {
        let vm = vm::VirtualMachine::new(vm::PySettings::default());
        let scope = vm.new_scope_with_builtins();
        let draw_cmds = add_render_methods(&vm, &scope).unwrap();
        let ast = vm.compile(START_STRING, Exec, "<start>".to_string()).unwrap();

        Ok(QuickTest {
            // text editing
            font: Asset::new(Font::load("font.ttf")),
            text: START_STRING.to_string(),
            cursor: START_STRING.len() - 1,
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

    fn update(&mut self, _: &mut Window) -> Result<()> {
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
                Err(err) => {
                    let mut out = Vec::new();
                    vm::exceptions::write_exception(&mut out, &self.vm, &err);
                    self.error = Some(String::from_utf8(out).unwrap());
                },
            };
            self.ast_dirty = false;
        }

        Ok(())
    }

    fn event(&mut self, event: &Event, _: &mut Window) -> Result<()> {
        use quicksilver::input::{
            ButtonState::Pressed,
            Key::{Back, Left, Return, Right},
        };
        match event {
            &Event::Typed(c) => {
                self.ast_dirty = true;
                self.type_char(c);
            }
            &Event::Key(k, Pressed) => {
                self.ast_dirty = true;
                match k {
                    Back => {
                        if self.text.len() > 0 {
                            self.cursor = self.cursor_left();
                            self.text.remove(self.cursor);
                            if self.cursor == self.text.len() {
                                self.cursor = self.cursor_left();
                            }
                        }
                    }
                    Return => self.type_char('\n'),
                    Left => self.cursor = self.cursor_left(),
                    Right => self.cursor = self.cursor_right(),
                    _ => {},
                }
            }
            _ => {}
        };

        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        // Remove any lingering artifacts from the previous frame
        window.clear(Color::BLACK)?;

        let mut color = Color::WHITE;
        let mut transforms = vec![quicksilver::geom::Transform::IDENTITY];

        match self.vm.run_code_obj(self.ast.clone(), self.scope.clone()) {
            Ok(_) => {}
            Err(err) => {
                // this really shouldn't happen, we should save asts that don't error.
                println!("bad ast slipped in!");
                let mut out = Vec::new();
                vm::exceptions::write_exception(&mut out, &self.vm, &err);
                self.error = Some(String::from_utf8(out).unwrap());
            }
        }
        while let Ok(cmd) = self.draw_cmds.try_recv() {
            use draw::DrawCommand::*;
            match cmd {
                Rect(rect) => window.draw_ex(&rect, Col(color), *transforms.last().unwrap(), 0),
                Fill(new_color) => color = new_color,
                Transform(t) => {
                    let now = transforms.last_mut().unwrap();
                    *now = *now * t;
                }
                PushTransform => {
                    let now = transforms.last().unwrap().clone();
                    transforms.push(now)
                }
                PopTransform => {
                    if transforms.len() >= 2 {
                        transforms.pop();
                    }
                }
            }
        }

        let mut text = self.text.clone();
        if self.timer % 40 < 25 {
            let i = self.cursor;

            let cursor_text = if text.chars().nth(i).unwrap() == '\n' {
                "_\n"
            } else {
                "_"
            };

            text.replace_range(i..=i, cursor_text);
        };

        let font = &mut self.font;
        let error = self.error.as_ref();

        font.execute(|font| {
            let img = font.render(text.as_str(), &FontStyle::new(24.0, Color::WHITE))?;
            window.draw(&img.area(), Img(&img));

            if let Some(error) = error {
                use quicksilver::geom::Shape;
                let img = font.render(error.as_str(), &FontStyle::new(24.0, Color::RED))?;
                window.draw(&img.area().translate((300, 0)), Img(&img));
            }

            Ok(())
        })?;

        Ok(())
    }
}

fn main() {
    run::<QuickTest>(
        "QuickTest",
        Vector::new(800, 600),
        Settings {
            multisampling: Some(16),
            scale: quicksilver::graphics::ImageScaleStrategy::Blur,
            ..Settings::default()
        },
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
        use quicksilver::geom::{Rectangle, Vector};

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
