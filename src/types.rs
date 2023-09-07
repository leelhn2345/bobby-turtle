#[derive(thiserror::Error, Debug)]
pub enum MyError {
    #[error(transparent)]
    RequestError(#[from] teloxide::RequestError),
}

pub type MyResult<T> = Result<T, MyError>;
