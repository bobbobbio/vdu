// copyright 2021 Remi Bernotavicius
use wasm_bindgen::prelude::*;
use web_sys::{Request, RequestInit, RequestMode, Response};
use vdu_path_tree::PathTree;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsCast;
use vdu::Vdu;
use std::cell::RefCell;
use std::rc::Rc;

mod vdu;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => ($crate::log(&format_args!($($t)*).to_string()))
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn canvas() -> web_sys::HtmlCanvasElement {
    let document = window().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap()
}

fn get_drawing_context(canvas: &web_sys::HtmlCanvasElement) -> web_sys::CanvasRenderingContext2d {
    canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap()
}

async fn load_path_tree() -> Result<PathTree, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init("./tree", &opts)?;

    request
        .headers()
        .set("Accept", "application/octet-stream")?;

    let window = window();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into().unwrap();
    let value = JsFuture::from(resp.array_buffer()?).await?;
    let array = js_sys::Uint8Array::new(&value);
    let buffer = array.to_vec();
    Ok(bincode::deserialize(&buffer[..]).unwrap())
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn set_up_rendering(vdu: Rc<RefCell<Vdu>>) {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        vdu.borrow_mut().render();

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn set_canvas_to_window_size(canvas: &web_sys::HtmlCanvasElement) {
    let document = window().document().unwrap();
    let elem = document.document_element().unwrap();
    let width = elem.client_width();
    let height = elem.client_height();

    canvas.set_width(width as u32 - 20);
    canvas.set_height(height as u32 - 20);
}

fn set_up_input(vdu: Rc<RefCell<Vdu>>) {
    let canvas = canvas();

    let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        vdu
            .borrow_mut()
            .on_mouse_move(event.offset_x() as f64, event.offset_y() as f64);
    }) as Box<dyn FnMut(_)>);

    canvas
        .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();
}

fn display_path_tree(tree: PathTree) -> Result<(), JsValue> {
    console_log!("path tree; {} nodes {} bytes", tree.size(), tree.num_bytes());

    set_canvas_to_window_size(&canvas());

    let closure = Closure::wrap(Box::new(move |_: JsValue| {
        let canvas = canvas();
        set_canvas_to_window_size(&canvas);
    }) as Box<dyn FnMut(_)>);
    window().add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())?;
    closure.forget();

    let drawing_context = get_drawing_context(&canvas());
    let vdu = Rc::new(RefCell::new(Vdu::new(
        drawing_context,
        canvas(),
        tree,
    )));

    set_up_rendering(vdu.clone());
    set_up_input(vdu.clone());

    Ok(())
}

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log!("VDU loading");

    let tree = load_path_tree().await?;
    display_path_tree(tree)?;

    Ok(())
}
