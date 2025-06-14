use thiserror::Error;

#[derive(Error, Debug)]
pub enum CharConversionError {
    #[error("Field `{0}` is out of bounds.")]
    FieldOob(String),
}
