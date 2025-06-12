// src/gemini_client.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value; // Menggunakan serde_json::Value untuk fleksibilitas input JSON
use std::fs::File; // Untuk membuka file
use std::io::BufReader; // Untuk membaca file secara efisien
use std::path::Path; // Untuk bekerja dengan path file

// Struct untuk merepresentasikan struktur request body ke Gemini API
#[derive(Serialize)]
struct GenerateContentRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

// Struct untuk mem-parsing struktur response dari Gemini API
#[derive(Deserialize, Debug)]
struct GenerateContentResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: ContentResponse,
}

#[derive(Deserialize, Debug)]
struct ContentResponse {
    parts: Vec<PartResponse>,
}

#[derive(Deserialize, Debug)]
struct PartResponse {
    text: String,
}

/// Klien untuk berinteraksi dengan Google Gemini API.
pub struct GeminiClient {
    api_key: String,
    http_client: Client,
    model_name: String,
}

impl GeminiClient {
    /// Membuat instance baru dari GeminiClient.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Kunci API Google AI Studio Anda.
    /// * `model_name` - Model yang akan digunakan, contoh: "gemini-1.5-flash-latest".
    pub fn new(api_key: String, model_name: String) -> Self {
        Self {
            api_key,
            http_client: Client::new(),
            model_name,
        }
    }

    /// Fungsi untuk membaca dan mem-parsing file JSON.
    /// Mengembalikan `Result` yang berisi data JSON (`Value`) atau sebuah Error.
    pub fn read_json_from_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        // 1. Buka file di path yang diberikan
        let file = File::open(path)?;
        // 2. Buat buffer reader untuk efisiensi
        let reader = BufReader::new(file);
        // 3. Parse JSON dari reader langsung ke dalam tipe serde_json::Value
        let json_data = serde_json::from_reader(reader)?;
        // 4. Kembalikan data jika berhasil
        Ok(json_data)
    }

    /// Mengirim data JSON ke Gemini untuk dianalisis.
    ///
    /// # Arguments
    ///
    /// * `json_content` - Data JSON yang akan dianalisis dalam bentuk `serde_json::Value`.
    /// * `prompt_instruction` - Instruksi spesifik untuk Gemini tentang apa yang harus dilakukan dengan data JSON tersebut.
    ///
    /// # Returns
    ///
    /// `Result<String, Box<dyn std::error::Error>>` yang berisi hasil analisis teks dari Gemini
    /// atau error jika terjadi masalah.
    pub async fn analyze_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        let api_url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.model_name
        );

        let json_path = format!("{}/transkrip.json", env!("CARGO_MANIFEST_DIR"));

        let data_laporan = match GeminiClient::read_json_from_file(json_path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("❌ Gagal membaca atau mem-parsing file JSON: {}", e);
                return Err(e);
            }
        };

        println!("✅ File JSON berhasil dibaca.");

        // 4. Tentukan instruksi yang jelas untuk Gemini
        let instruksi: String = format!("{}  saya adalah seorang pemuda content creator tiktok dan saya ingin kamu menganalisis data transkrip video saya. saya ingin anda mencari 4 part paling menarik dan paling berpotensi di sukai banak orang terutama pemuda. berikan juka hooknya agar lebih menarik pengguna tiktok. pastikan jawaban kamu hanya response json saja tidak di tambahi apapun berikan reponse berupa json yang berformat seperti ini

        {{
            start: 100,
            end: 200,
            hook: 'hook text',
        
    }}", data_laporan);

        println!("Mengirim data JSON ke Gemini untuk analisis...");
        println!("-------------------------------------------------");

        // Membangun request body sesuai format yang dibutuhkan Gemini API
        let request_body = GenerateContentRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: instruksi.to_string(),
                }],
            }],
        };

        // Mengirim request ke API
        let response = self
            .http_client
            .post(&api_url)
            .query(&[("key", &self.api_key)])
            .json(&request_body)
            .send()
            .await?;

        // Mengecek jika request gagal
        if !response.status().is_success() {
            let error_body = response.text().await?;
            return Err(format!("Error dari API: {}", error_body).into());
        }

        // Parsing response JSON yang berhasil
        let response_body = response.json::<GenerateContentResponse>().await?;

        // Mengekstrak teks dari response yang kompleks
        if let Some(candidate) = response_body.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }

        Err("Tidak dapat menemukan konten teks dalam respons API.".into())
    }
}
