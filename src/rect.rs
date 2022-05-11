//! A container for Rectangle structure.
#[derive(Clone, Copy, Debug)]
/// Rectangle structure
pub struct Rect<T> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

impl Rect<i32> {
    /// Whether the rectangle is empty or not.
    pub fn is_empty(&self) -> bool {
        self.left == 0 && self.top == 0 && self.right == 0 && self.bottom == 0
    }
}

impl From<ul_sys::ULRect> for Rect<f32> {
    fn from(r: ul_sys::ULRect) -> Self {
        Rect {
            left: r.left,
            top: r.top,
            right: r.right,
            bottom: r.bottom,
        }
    }
}

impl From<ul_sys::ULIntRect> for Rect<i32> {
    fn from(r: ul_sys::ULIntRect) -> Self {
        Rect {
            left: r.left,
            top: r.top,
            right: r.right,
            bottom: r.bottom,
        }
    }
}

impl From<Rect<f32>> for ul_sys::ULRect {
    fn from(r: Rect<f32>) -> Self {
        ul_sys::ULRect {
            left: r.left,
            top: r.top,
            right: r.right,
            bottom: r.bottom,
        }
    }
}

impl From<Rect<i32>> for ul_sys::ULIntRect {
    fn from(r: Rect<i32>) -> Self {
        ul_sys::ULIntRect {
            left: r.left,
            top: r.top,
            right: r.right,
            bottom: r.bottom,
        }
    }
}
