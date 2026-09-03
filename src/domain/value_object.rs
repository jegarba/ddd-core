/// Todo Value Object se construye validado o no se construye — `Self`
/// nunca existe en un estado inválido, por eso `new` es la única vía pública.
pub trait ValueObject: Sized {
    type Raw;
    type Error;
    fn new(raw: Self::Raw) -> Result<Self, Self::Error>;
}
