use std::hint::black_box;

use ul_next::Library;

#[test]
fn load_test_appcore() {
    let lib = unsafe { Library::load_with_appcore().unwrap() };

    black_box(lib);
}
