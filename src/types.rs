#[derive(thiserror::Error, Debug)]
pub enum MyError {
    #[error(transparent)]
    RequestError(#[from] teloxide::RequestError),
    // for `return Err("wtf is this error")`
    // #[error("unknown error: {0}")]
    // Unknown(String),
}

pub type MyResult<T> = Result<T, MyError>;
