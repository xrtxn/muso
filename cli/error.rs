use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum Error {
    #[error("Path {path} is not valid as root folder!")]
    InvalidRoot { path: String },
}
