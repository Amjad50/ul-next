use std::string::FromUtf8Error;

/// Errors can occure when creating some of the structs.
#[derive(Debug, thiserror::Error)]
pub enum CreationError {
    /// Ultralight library returned null pointer, and couldn't create the
    /// object.
    #[error("Creation of an object failed because Ultralight returned a null pointer")]
    NullReference,
    /// Ultralight returned null pointer when trying to create an Ultralight string
    /// from a Rust string.
    #[error("Failed to convert the string {0} to an ultralight string")]
    UlStringCreationError(String),
    /// Ultralight string contained invalid UTF-8, and couldn't convert it to a
    /// valid Rust string.
    #[error("Failed to convert an ultralight string to Rust string")]
    RustStringCreationError(#[from] FromUtf8Error),
    /// `&str` contained a null byte, and couldn't convert it to a valid C string without losing data.
    #[error("Failed to convert a rust `&str` to a C string")]
    CStringCreationError(#[from] std::ffi::NulError),
}
