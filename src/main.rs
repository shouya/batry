use crate::error::Result;
use app::App;

mod app;
mod error;
mod state;
mod upower;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
  let app = App::new_from_env().await?;
  app.run().await
}
