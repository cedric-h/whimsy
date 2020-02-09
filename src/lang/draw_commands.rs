use crate::draw::DrawCommand;

impl Lang {
    /// Inserts draw commands and returns a Receiver where the commands are sent.
    fn draw_commands(&self) -> vm::pyobject::PyResult<mpsc::Receiver<DrawCommand>> {
        use quicksilver::geom::Transform as Matrix;
        use DrawCommand::*;
        use quicksilver::graphics::Color;
        use vm::function::OptionalArg;
        use vm::obj::objfloat::IntoPyFloat as Num;
        use vm::obj::objstr::PyStringRef;
        use vm::pyobject::Either;

        let (draws_tx, draws_rx) = mpsc::channel();

        macro_rules! expose_fn {
            ( $name:literal, $tx:ident, $fn:expr $(,)? ) => {
                self.scope.globals.set_item(
                    $name,
                    self.vm.context().new_function({
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

        expose_fn!(
            "wave",
            _d,
            move |vm: &vm::VirtualMachine, args: vm::function::PyFuncArgs| -> vm::pyobject::PyResult {
                use vm::pyobject::{TryFromObject, IntoPyObject};

                let pymilliseconds: Num = args.bind(vm)?;
                let ms = pymilliseconds.to_f64();
                let pytime_raw = vm
                    .current_scope()
                    .globals
                    .get_item_option("time", vm)?
                    .expect("no time variable found");
                let time_raw: f64 = TryFromObject::try_from_object(vm, pytime_raw)?;

                (time_raw*(std::f64::consts::PI/ms)).sin().abs().into_pyobject(vm)
            },
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
        expose_fn!("push", d, move || d.send(PushTransform).unwrap())?;
        expose_fn!("pop", d, move || d.send(PopTransform).unwrap())?;

        return Ok(draws_rx);
    }
}
