//! LLM 連携（Gemini API）
//!
//! Google Gemini API を使って週次ダイエットレポートを生成する。
//! API キーは環境変数 `GEMINI_API_KEY` から読み込む（`.env` ファイルで設定）。
//!
//! # 呼び出しフロー
//! 1. `app.rs` の `start_report()` が別スレッドを生成
//! 2. そのスレッド内で `generate_weekly_report(prompt)` を呼ぶ
//! 3. 結果を mpsc チャネル経由でメインスレッドへ返す

use anyhow::{anyhow, Result};

/// Gemini API のエンドポイントテンプレート。
/// `{key}` の部分に実際の API キーを埋め込む。
const GEMINI_ENDPOINT: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=";

/// 環境変数 `GEMINI_API_KEY` を読み込み、Gemini API でレポートを生成する。
///
/// # Errors
/// - `GEMINI_API_KEY` が未設定の場合
/// - HTTP リクエストが失敗した場合
/// - レスポンスのパースに失敗した場合
pub fn generate_weekly_report(prompt: &str) -> Result<String> {
    generate_text(prompt, 1024)
}

/// トレーニングメモを、週次評価に使いやすい短い要約へ変換する。
pub fn summarize_training_memos(entries: &[(String, String)]) -> Result<Vec<(String, String)>> {
    if entries.is_empty() {
        return Ok(Vec::new());
    }

    let mut prompt = String::from(
        "以下は筋トレ日の自由メモです。各日の内容を、週次の評価に使いやすい客観的で短い日本語に要約してください。\n\
         助言や励ましは不要です。パフォーマンスに関する情報は落とさずに内容を圧縮するようにしてください。出力は'YYYY-MM-DD: 要約'という形で
         冒頭にタイムスタンプをつけるようにしてください。\n\n",
    );

    for (date, memo) in entries {
        prompt.push_str(&format!("{}: {}\n", date, memo.trim()));
    }

    let text = generate_text(&prompt, 512)?;
    let mut result = Vec::new();

    for line in text.lines() {
        let Some((date, summary)) = line.split_once(':') else { continue; };
        let date = date.trim().to_string();
        let summary = summary.trim().to_string();
        if !date.is_empty() && !summary.is_empty() {
            result.push((date, summary));
        }
    }

    Ok(result)
}

fn generate_text(prompt: &str, max_output_tokens: u32) -> Result<String> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .map_err(|_| anyhow!("GEMINI_API_KEY が設定されていません。.env ファイルを確認してください。"))?;

    if api_key.trim().is_empty() || api_key == "your_gemini_api_key_here" {
        return Err(anyhow!("有効な GEMINI_API_KEY を .env ファイルに設定してください。"));
    }

    let url = format!("{}{}", GEMINI_ENDPOINT, api_key);

    let body = serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": [{ "text": prompt }]
        }],
        "generationConfig": {
            "maxOutputTokens": max_output_tokens
        }
    });

    let response = ureq::post(&url)
        .set("content-type", "application/json")
        .send_json(&body)
        .map_err(|e| anyhow!("Gemini API リクエスト失敗: {}", e))?;

    let json: serde_json::Value = response
        .into_json()
        .map_err(|e| anyhow!("Gemini API レスポンスのパース失敗: {}", e))?;

    // Gemini レスポンス構造: candidates[0].content.parts[0].text
    json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Gemini API からテキストを取得できませんでした。レスポンス: {}", json))
}
