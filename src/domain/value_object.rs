/// Every Value Object is constructed validated or not at all — `Self` never
/// exists in an invalid state, so `new` is the only public entry point.
pub trait ValueObject: Sized {
    type Raw;
    type Error;
    fn new(raw: Self::Raw) -> Result<Self, Self::Error>;
}
