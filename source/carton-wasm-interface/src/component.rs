pub use carton_wasm::lib::types::{Dtype, TensorNumeric, TensorString};

wit_bindgen::generate!({
    world: "model",
    path: "../wit",
    exports: {
        world: Model
    }
});

static mut INFER_FN: Option<fn(Vec<(String, Tensor)>) -> Vec<(String, Tensor)>> = None;

pub fn set_infer_fn(f: fn(Vec<(String, Tensor)>) -> Vec<(String, Tensor)>) {
    unsafe {
        INFER_FN = Some(f);
    }
}

struct Model;

impl Guest for Model {
    fn infer(in_: Vec<(String, Tensor)>) -> Vec<(String, Tensor)> {
        unsafe {
            match INFER_FN {
                Some(f) => f(in_),
                None => panic!("Infer function not set"),
            }
        }
    }
}