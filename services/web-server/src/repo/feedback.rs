use std::ops::Deref;

use super::Repo;
use anyhow::Result;
use chrono::NaiveDateTime;

#[derive(Debug, sqlx::FromRow, Default)]
pub struct Feedback {
    pub id: i32,
    pub text: String,
    pub image_name: String,
    pub model: String,
    pub user_feedback: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl Repo {
    pub async fn create_feedback(
        &self,
        text: String,
        image_name: String,
        model: String,
        feedback: i32,
    ) -> Result<i32> {
        let client = self.db_pool.deref();
        let saved_feedback = sqlx::query_as!(
            Feedback,
            r#"
            INSERT INTO feedback (text, image_name, model, user_feedback) 
            VALUES ($1, $2, $3, $4) 
            RETURNING *"#,
            text,
            image_name,
            model,
            feedback,
        )
        .fetch_one(client.deref())
        .await?;

        Ok(saved_feedback.id)
    }
}
