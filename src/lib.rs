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
    lifecycle::{run, Event, EventStream, Settings, Window},
    Result,
};

mod lang;

const START_STRING: &'static str = r#""#;

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

macro_rules! handle {
    ( $event_target:expr, $kind:literal, $handler:expr ) => {
        let handler = Closure::wrap(Box::new($handler) as Box<dyn FnMut(web_sys::CustomEvent)>);
        (&$event_target)
            .add_event_listener_with_callback($kind, handler.as_ref().unchecked_ref())
            .expect(concat!("can't listen to '", $kind, "' events on EventTarget passed"));
        // forget the handler to keep it alive
        handler.forget();
    }
}

fn now() -> f64 {
    web_sys::window()
        .expect("Couldn't lock the window object")
        .performance()
        .expect("Could not get performance timer")
        .now()
}

fn err_event(err: String) -> web_sys::CustomEvent {
    let e = web_sys::CustomEvent::new("error").unwrap();

    e.init_custom_event_with_can_bubble_and_cancelable_and_detail(
        "error", true, true, &err.into(),
        );

    e
}

async fn app(win: Window, mut gfx: Graphics, mut events: EventStream, new_code_events: web_sys::EventTarget, to_editor: web_sys::EventTarget) -> Result<()> {
    use wasm_bindgen::JsCast;
    
    let mut source_code = String::new();
    let mut ast_dirty = true;

    let mut lang = Lang::new();
    let draw_cmds = lang.draw_commands();

    // listen for new code
    let (new_code_tx, new_code_rx) = mpsc::channel();
    handle!(&event_target, "code", move |e: web_sys::CustomEvent| {
        let code = e.detail().as_string().expect("The detail field of the 'code' event must be a string");
        new_code_tx.send(code).expect("Couldn't send new code");
    });

    if let Err(e) = lang.run_ast() {
        log::error!("bad ast slipped in!");
        to_editor.dispatch_event(err_event(e)).unwrap();
    }
    let start = now();
    loop {
        while let Some(ev) = events.next_event().await {}

        while let Ok(new_code) = new_code_rx.try_recv() {
            if let Err(e) = lang.update_ast(&new_code) {
                to_editor.dispatch_event(err_event(e)).unwrap();
            }
            ast_dirty = true;
        }

        lang.set_global("time", now() - start);
        draw(&mut gfx, &draw_cmds)?;
        gfx.present(&win)?;
    }
}

fn draw(gfx: &mut Graphics, draw_cmds: &mpsc::Receiver<draw::DrawCommand>) -> Result<()> {
    use quicksilver::graphics::Color;

    // Remove any lingering artifacts from the previous frame
    gfx.clear(Color::BLACK);

    let mut color = Color::WHITE;
    let mut transforms = vec![quicksilver::geom::Transform::IDENTITY];

    while let Ok(cmd) = draw_cmds.try_recv() {
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

#[wasm_bindgen]
pub fn main(x: f32, y: f32, new_code_events: wasm_bindgen::JsValue, to_editor: wasm_bindgen::JsValue) {

    // make sure the event listeners are of the right type, and fail fast if they aren't.
    let to_editor = to_editor
        .dyn_into::<web_sys::EventTarget>()
        .expect("to_editor must be an EventTarget");
    let event_target = new_code_events
        .dyn_into::<web_sys::EventTarget>()
        .expect("third variable passed to main not an EventTarget");

    run(
        Settings {
            size: quicksilver::geom::Vector::new(x, y).into(),
            title: "QuickTest",
            multisampling: Some(16),
            ..Settings::default()
        },
        move |w, g, e| app(w, g, e, new_code_events, to_editor)
    );
}

