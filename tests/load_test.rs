use std::hint::black_box;

use ul_next::Library;

#[test]
fn load_test() {
    let lib = unsafe { Library::load().unwrap() };

    black_box(lib);
}
