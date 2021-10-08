mod backbone;
mod core;
mod index;

// Important note: all data stored on disk by Emdrive is big-endian.
// Use `from_be_bytes` and `to_be_bytes` methods, with `array_ref!` if needed.
