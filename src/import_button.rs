use std::sync::Mutex;

type Rect = [f32; 4];

static WANTED: Mutex<Option<Rect>> = Mutex::new(None);
static SENT: Mutex<Option<Rect>> = Mutex::new(None);

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    fn ume_set_import_rect(x: f32, y: f32, w: f32, h: f32);
}

pub fn place(x: f32, y: f32, w: f32, h: f32) {
    if let Ok(mut wanted) = WANTED.lock() {
        *wanted = Some([x, y, w, h]);
    }
}

pub fn flush() {
    let wanted = WANTED.lock().ok().and_then(|mut w| w.take());
    let Ok(mut sent) = SENT.lock() else { return };
    if *sent == wanted {
        return;
    }
    *sent = wanted;
    #[cfg(target_arch = "wasm32")]
    {
        let [x, y, w, h] = wanted.unwrap_or([0.0; 4]);
        ume_set_import_rect(x, y, w, h);
    }
}
