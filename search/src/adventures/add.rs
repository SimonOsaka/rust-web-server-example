use extra::meilisearch::operation::{add_documents, del_documents, MeiliSearchStatus};
use vars::MyAdventures;

use super::error::SearchError;

#[tracing::instrument]
pub async fn add_adventure(ad: MyAdventures) -> Result<bool, SearchError> {
    add_adventures(vec![ad]).await
}

#[tracing::instrument]
pub async fn add_adventures(ads: Vec<MyAdventures>) -> Result<bool, SearchError> {
    let status = add_documents(ads).await?;

    match status {
        MeiliSearchStatus::Succeeded => return Ok(true),
        _ => return Ok(false),
    };
}

#[tracing::instrument]
pub async fn delete_adventure(uid: i64) -> Result<bool, SearchError> {
    if uid < 1 {
        return Ok(false);
    }

    let status = del_documents(vec![uid]).await?;

    match status {
        MeiliSearchStatus::Succeeded => return Ok(true),
        _ => return Ok(false),
    };
}

#[tracing::instrument]
pub async fn delete_adventures(uids: Vec<i64>) -> Result<bool, SearchError> {
    let status = del_documents(uids).await?;

    match status {
        MeiliSearchStatus::Succeeded => return Ok(true),
        _ => return Ok(false),
    };
}