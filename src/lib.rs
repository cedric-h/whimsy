// RustPython
pub use rustpython_compiler as compiler;
pub use rustpython_vm as vm;
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

mod draw;
mod lang;

macro_rules! handle {
    ( $event_target:expr, $kind:literal, $handler:expr ) => {
        use wasm_bindgen::JsCast;
        let handler = Closure::wrap(Box::new($handler) as Box<dyn FnMut(web_sys::CustomEvent)>);
        (&$event_target)
            .add_event_listener_with_callback($kind, handler.as_ref().unchecked_ref())
            .expect(concat!(
                "can't listen to '",
                $kind,
                "' events on EventTarget passed"
            ));
        // forget the handler to keep it alive
        handler.forget();
    };
}

fn err_event(err: String) -> web_sys::CustomEvent {
    let e = web_sys::CustomEvent::new("error").unwrap();
    e.init_custom_event_with_can_bubble_and_cancelable_and_detail("error", true, true, &err.into());
    e
}

async fn app(
    win: Window,
    mut gfx: Graphics,
    mut events: EventStream,
    from_editor: web_sys::EventTarget,
    to_editor: web_sys::EventTarget,
) -> Result<()> {
    // grab ahold of the web browser's performance object for reading the time
    let time = web_sys::window()
        .expect("Couldn't lock the window object")
        .performance()
        .expect("Couldn't get the performance object from the window");

    // listen for new code
    let (new_code_tx, new_code_rx) = mpsc::channel();
    handle!(&from_editor, "code", move |e: web_sys::CustomEvent| {
        let code = e
            .detail()
            .as_string()
            .expect("The detail field of the 'code' event must be a string");
        new_code_tx.send(code).expect("Couldn't send new code");
    });

    // instantiate our wrapper around the rustpython VM
    let mut lang = lang::Lang::new();
    let draw_cmds = lang
        .draw_commands()
        .unwrap_or_else(|e| panic!("failed to add drawing library: {}", e));


    let start = time.now();
    loop {
        while let Some(_) = events.next_event().await {}

        while let Ok(new_code) = new_code_rx.try_recv() {
            if let Err(e) = lang.try_update_ast(&new_code) {
                to_editor.dispatch_event(&err_event(e)).unwrap();
            }
        }
        if let Err(e) = lang.run_ast() {
            log::error!("the live AST threw an error");
            to_editor.dispatch_event(&err_event(e)).unwrap();
        }

        lang.set_global("time", time.now() - start);
        draw::render_commands(&mut gfx, &draw_cmds);
        gfx.present(&win)?;
    }
}

#[wasm_bindgen]
pub fn main(
    x: f32,
    y: f32,
    from_editor_raw: wasm_bindgen::JsValue,
    to_editor_raw: wasm_bindgen::JsValue,
) {
    use wasm_bindgen::JsCast;

    // make sure the event listeners are of the right type, and fail fast if they aren't.
    let to_editor = to_editor_raw
        .dyn_into::<web_sys::EventTarget>()
        .expect("to_editor parameter to 'main' must be an EventTarget");
    let from_editor = from_editor_raw
        .dyn_into::<web_sys::EventTarget>()
        .expect("from_editor parameter to 'main' must be an EventTarget");

    run(
        Settings {
            size: quicksilver::geom::Vector::new(x, y).into(),
            title: "QuickTest",
            multisampling: Some(16),
            ..Settings::default()
        },
        move |w, g, e| app(w, g, e, from_editor, to_editor),
    );
}
