
use xlib::client::PostgresClient;
use std::sync::Arc;

mod feedback;
pub struct Repo {
    db_pool: Arc<PostgresClient>,
}

impl Repo {
    pub fn new(db_pool: Arc<PostgresClient>) -> Self {
        Self { db_pool }
    }
}

