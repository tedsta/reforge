use std::ops::{Deref, DerefMut};
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

pub struct NoEncode<T> {
    inner: Option<T>,
}

impl<T> NoEncode<T> {
    fn new(inner: T) -> NoEncode<T> {
        NoEncode {
            inner: Some(inner),
        }
    }
    
    fn empty() -> NoEncode<T> {
        NoEncode {
            inner: None,
        }
    }
    
    fn unwrap(self) -> T {
        self.inner.expect("Failed to unwrap uninitialized NoEncode")
    }
}

impl<T> Deref for NoEncode<T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        self.inner.as_ref().expect("Failed to mutably dereference uninitialized NoEncode")
    }
}
impl<T> DerefMut for NoEncode<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        self.inner.as_mut().expect("Failed to mutably dereference uninitialized NoEncode")
    }
}

impl<T> Decodable for NoEncode<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<NoEncode<T>, D::Error> { Ok(NoEncode::empty()) }
}

impl<T> Encodable for NoEncode<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> { Ok(()) }
}