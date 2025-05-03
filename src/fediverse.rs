use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::errors::ResponseError;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct NoteRequest {
    #[serde(rename = "noteId")]
    note_id: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct NoteResponse {
    pub id: String,
    #[serde(rename = "repliesCount")]
    pub replies_count: u64,
    #[serde(rename = "reactionCount")]
    pub reaction_count: u64,
}

pub async fn note(host: String, note_id: String) -> Result<NoteResponse, ResponseError> {
    log::debug!("fn: note");

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{host}/api/notes/show"))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&NoteRequest { note_id })?)
        .send()
        .await?
        .json::<NoteResponse>()
        .await?;

    Ok(response)
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct NotesUserRequest {
    #[serde(rename = "userId")]
    user_id: String,
    limit: u32,
}

pub type NotesUserResponse = HashMap<String, NoteResponse>;

pub async fn notes_user(host: String, user_id: String) -> Result<NotesUserResponse, ResponseError> {
    log::debug!("fn: notes_user");

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{host}/api/users/notes"))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&NotesUserRequest {
            user_id,
            limit: 100,
        })?)
        .send()
        .await?
        .json::<Vec<NoteResponse>>()
        .await?;

    let mut hashmap = NotesUserResponse::new();

    for item in response {
        hashmap.insert(item.id.clone(), item);
    }

    Ok(hashmap)
}
