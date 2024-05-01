use miette::IntoDiagnostic;
use sqlx::SqliteConnection;

use self::{category::Category, persisted_event::PersistedEvent, producer::Producer, tag::Tag};

use miette::Result;

pub mod category;
pub mod persisted_event;
pub mod producer;
pub mod tag;

#[derive(Default)]
pub struct Events {
    persisted_events: Vec<PersistedEvent>,
    tags: Vec<Tag>,
    categories: Vec<Category>,
    producers: Vec<Producer>,
}

impl Events {
    pub fn persisted_events(&self) -> &Vec<PersistedEvent> {
        &self.persisted_events
    }

    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }

    pub fn categories(&self) -> &Vec<Category> {
        &self.categories
    }

    pub async fn load(&mut self, connection: &mut SqliteConnection) -> Result<()> {
        self.load_tags(connection).await?;

        Ok(())
    }

    pub async fn load_tags(&mut self, connection: &mut SqliteConnection) -> Result<()> {
        let tag_records = sqlx::query("SELECT * FROM tag_descriptions")
            .fetch_all(connection)
            .await
            .into_diagnostic()?;

        for tag_record in tag_records {
            let tag = Tag::from_sql_row(tag_record)?;
            self.tags.push(tag);
        }

        Ok(())
    }

    pub async fn load_producers(&mut self, connection: &mut SqliteConnection) -> Result<()> {
        let producer_records = sqlx::query("SELECT * FROM producers")
            .fetch_all(connection)
            .await
            .into_diagnostic()?;

        for producer_record in producer_records {
            let producer = Producer::from_sql_row(producer_record)?;
            self.producers.push(producer);
        }

        Ok(())
    }

    pub async fn load_categories(&mut self, connection: &mut SqliteConnection) -> Result<()> {
        let category_records = sqlx::query("SELECT * FROM categories")
            .fetch_all(connection)
            .await
            .into_diagnostic()?;

        for category_record in category_records {}

        Ok(())
    }
}
