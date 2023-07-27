#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),

  #[error(transparent)]
  Parse(#[from] envy::Error),

  #[error(transparent)]
  DBus(#[from] zbus::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
