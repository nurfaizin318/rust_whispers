use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Debug)]
struct RequestBody {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct ResponseBody {
    choices: Vec<Choice>,
}

pub struct OpenAIClient {
    client: Client,
    api_key: String,
}

impl OpenAIClient {
    pub fn new() -> Self {
        let api_key = String::from("9dba65aef5f64cca8cd7be5a80be7c4d");
        OpenAIClient {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn analyze_transcript(&self, transcript_json: &Value) -> Result<String, Box<dyn std::error::Error>> {
        let content = format!(
            "Saya punya transkrip video dalam format JSON:\n{}\nTolong identifikasi 3-5 bagian paling menarik. Sertakan alasan dan timestamp.",
            serde_json::to_string_pretty(transcript_json)?
        );

        let messages = vec![
            Message {
                role: "user".to_string(),
                content: "Kamu adalah analis konten video. Tugasmu adalah menemukan bagian paling menarik dari sebuah transkrip video.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content,
            },
        ];

        let request_body = RequestBody {
            model: "google/gemma-3n-e4b-it".to_string(), // atau "gpt-3.5-turbo"
            messages,
        };

        let res = self.client
            .post("https://api.aimlapi.com/v1")
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await?;

        let raw_text = res.text().await?;
        println!("🔍 Raw response:\n{}", raw_text);

        let response_body: ResponseBody = serde_json::from_str(&raw_text)?;

        println!("🧾 Response dari OpenAI:\n{:#?}", response_body);

        Ok(response_body.choices[0].message.content.clone())
    }
}
