use serde::{Deserialize, Serialize};

use crate::errors::ResponseError;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct NoteRequest {
    #[serde(rename = "noteId")]
    note_id: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct NoteResponse {
    #[serde(rename = "repliesCount")]
    pub replies_count: u64,
    #[serde(rename = "reactionCount")]
    pub reaction_count: u64,
}

pub async fn note(host: String, note_id: String) -> Result<NoteResponse, ResponseError> {
    println!("fn: fediverse::note");

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
