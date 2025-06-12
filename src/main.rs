    mod extractor;

    use extractor::ExtractAudio;

    mod gemini;

    use gemini::GeminiClient;
    use std::env;
    use dotenv::dotenv;

    #[tokio::main]
    async fn main() {

        dotenv().ok();

        let args: Vec<String> = env::args().collect();

        if args.len() < 2 {
            eprintln!("Usage: {} <input_video>", args[0]);
            std::process::exit(1);
        }

        let input_video = &args[1];

        let model_path = format!("{}/src/models/ggml-medium.bin", env!("CARGO_MANIFEST_DIR"));
        let output_wav = format!("{}/src/output.wav", env!("CARGO_MANIFEST_DIR"));
    

        let extractor = ExtractAudio::new(&model_path, input_video, &output_wav);

        extractor.convert_to_wav();
        extractor.transcribe();
        match extractor.convert_srt_to_ass() {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Error converting SRT to ASS: {}", e);
                std::process::exit(1);
            }
        }

        // match extractor.convert_video_to_tiktok_format() {
        //     Ok(_) => {},
        //     Err(e) => {
        //         eprintln!("Error converting video to TikTok format: {}", e);
        //         std::process::exit(1);
        //     }
        // }

        let api_key = match env::var("GEMINI_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                eprintln!("Error: Environment variable GEMINI_API_KEY tidak ditemukan.");
                eprintln!("Silakan set: export GEMINI_API_KEY='kunci_api_anda'");
                return;
            }
        };

        let client = GeminiClient::new(api_key, "gemini-1.5-flash-latest".to_string());

        match client.analyze_json().await {
            Ok(hasil_analisis) => {
                println!("✅ Hasil Analisis dari Gemini:\n");
                println!("{}", hasil_analisis);
            }
            Err(e) => {
                eprintln!("❌ Terjadi error saat menghubungi Gemini: {}", e);
            }
        }
    }
