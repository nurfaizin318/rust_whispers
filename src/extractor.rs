use std::fs::File;
use std::io::Write;

use indicatif::ProgressBar;
use serde_json::json;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct ExtractAudio<'a> {
    pub model_path: &'a str,

    pub output_wav: &'a str,
}

impl<'a> ExtractAudio<'a> {
    pub fn new(model_path: &'a str, output_wav: &'a str) -> Self {
        Self {
            model_path,

            output_wav,
        }
    }

    // pub fn convert_to_wav(&self) {
    //     println!("🔄 Mengonversi video ke WAV...");

    //     let mut child = Command::new("ffmpeg")
    //         .args([
    //             "-i",
    //             self.input_video,
    //             "-ar",
    //             "16000",
    //             "-ac",
    //             "1",
    //             "-y",
    //             self.output_wav,
    //         ])
    //         .stderr(Stdio::piped())
    //         .spawn()
    //         .expect("Gagal menjalankan ffmpeg");

    //     let stderr = child.stderr.take().expect("Tidak bisa baca stderr");
    //     let reader = BufReader::new(stderr);

    //     for line in reader.lines().flatten() {
    //         if line.contains("time=") {
    //             print!("\r⏳ {}", line);
    //             std::io::stdout().flush().unwrap();
    //         }
    //     }

    //     let status = child.wait().expect("Gagal menunggu ffmpeg selesai");
    //     if !status.success() {
    //         panic!("❌ Konversi video ke WAV gagal.");
    //     }

    //     println!("\n✅ Konversi selesai.");
    // }

    pub fn transcribe(&self) {
        println!("📥 Memuat model Whisper...");

        // load a context and model
        let ctx_params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(self.model_path, ctx_params).unwrap_or_else(|e| {
            eprintln!("❌ Gagal memuat model Whisper: {}", e);
            std::process::exit(1);
        });

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("id")); // Bahasa Indonesia

        let audio_data = Self::read_wav_to_pcm_f32(self.output_wav).expect("Gagal memuat file WAV");

        let mut state = ctx.create_state().expect("failed to create state");
        
        state
            .full(params, &audio_data[..])
            .expect("failed to run model");

        let num_segments = state
            .full_n_segments()
            .expect("failed to get number of segments");

        let bar = ProgressBar::new(num_segments as u64);

        let mut segments = vec![];

        for i in 0..num_segments {
            bar.inc(1);

            let segment = state
                .full_get_segment_text(i)
                .expect("failed to get segment");
            let start_timestamp = state
                .full_get_segment_t0(i)
                .expect("failed to get segment start timestamp");
            let end_timestamp = state
                .full_get_segment_t1(i)
                .expect("failed to get segment end timestamp");

            segments.push(json!({
                "start": start_timestamp,
                "end": end_timestamp,
                "text": segment
            }));
        }

        bar.finish_with_message("✅ Transkripsi selesai!");

        // Simpan ke file JSON
        let output_json = json!({ "segments": segments });
        let mut json_file = File::create("transkrip.json").expect("Gagal membuat file JSON");
        json_file
            .write_all(
                serde_json::to_string_pretty(&output_json)
                    .unwrap()
                    .as_bytes(),
            )
            .expect("Gagal menulis ke file JSON");

        println!("📄 Disimpan ke: transkrip.json");

        // Buat file SRT
        let mut srt_file = File::create("transkrip.srt").expect("Gagal membuat file SRT");
        for (i, seg) in output_json["segments"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let start = Self::format_timestamp(seg["start"].as_f64().unwrap());
            let end = Self::format_timestamp(seg["end"].as_f64().unwrap());
            let text = seg["text"].as_str().unwrap();

            writeln!(srt_file, "{}\n{} --> {}\n{}\n", i + 1, start, end, text)
                .expect("Gagal menulis ke file SRT");
        }
    }
    fn read_wav_to_pcm_f32(path: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let mut reader = hound::WavReader::open(path)?;
        let spec = reader.spec();
        if spec.channels != 1 {
            return Err("Audio harus mono (1 channel)".into());
        }
        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => reader.samples::<f32>().map(|s| s.unwrap()).collect(),
            hound::SampleFormat::Int => {
                let max_value = 2_i32.pow(spec.bits_per_sample as u32 - 1) as f32;
                reader
                    .samples::<i32>()
                    .map(|s| s.unwrap() as f32 / max_value)
                    .collect()
            }
        };
        Ok(samples)
    }

    fn format_timestamp(timestamp: f64) -> String {
        let total_millis = (timestamp * 10.0) as u64; // jika timestamp satuan = 100 = 1 detik
        let hours = total_millis / 3_600_000;
        let minutes = (total_millis % 3_600_000) / 60_000;
        let seconds = (total_millis % 60_000) / 1000;
        let millis = total_millis % 1000;

        format!("{:02}:{:02}:{:02},{:03}", hours, minutes, seconds, millis)
    }
}
