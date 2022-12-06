use mongodb::{error::Error, Client};

/// Connects to the database and returns the database client.
pub async fn start_database() -> Result<Client, Error> {
    // IMPORTANT:
    // Define the database URI here.
    Client::with_uri_str("mongodb://db.example.com:12345").await
}
