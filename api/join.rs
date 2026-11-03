//! Fonction Vercel `/api/join` (voir docs/CONTRACT.md). Toute la logique est dans la
//! bibliothèque `spacejo_api` : ce fichier ne fait que la brancher sur le runtime.
use spacejo_api::{Endpoint, vercel};

#[tokio::main]
async fn main() -> Result<(), vercel_runtime::Error> {
    vercel::serve(Endpoint::Join).await
}
