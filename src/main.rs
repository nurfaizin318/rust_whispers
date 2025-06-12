mod extractor;

use extractor::ExtractAudio;

mod open_ai_client;

use serde_json::Value;
use std::fs;

use crate::open_ai_client::OpenAIClient;

#[tokio::main]
async fn main() {
    let model_path = "/Users/nurfaizin/Development/Rust/whisper/src/models/ggml-medium.bin";
    let output_wav = "/Users/nurfaizin/Development/Rust/whisper/src/output.wav";

    let extractor = ExtractAudio::new(model_path, output_wav);

    // extractor.convert_to_wav();
    extractor.transcribe();

    // let data = fs::read_to_string("transkrip.json").expect("Gagal membaca file JSON");
    // let json_data: Value = serde_json::from_str(&data).expect("Format JSON tidak valid");

    // let client = OpenAIClient::new();

    // match client.analyze_transcript(&json_data).await {
    //     Ok(result) => {
    //         println!("✅ Analisis bagian menarik:\n{}", result);
    //     }
    //     Err(e) => {
    //         eprintln!("❌ Gagal mengirim ke OpenAI: {}", e);
    //     }
    // }
}
