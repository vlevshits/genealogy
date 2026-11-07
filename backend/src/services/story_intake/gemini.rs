use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use serde_json::json;
use std::collections::HashMap;

use crate::{
    error::AppError,
    services::privacy::{DraftExtractedDataDto, DraftPersonDto},
};

/// Transcribes spoken audio bytes using Google Gemini generateContent with inline audio data.
pub async fn transcribe_audio_with_gemini(
    api_key: &str,
    model: &str,
    audio_bytes: &[u8],
    mime_type: &str,
    lang_hint: Option<&str>,
) -> Result<String, AppError> {
    if api_key.trim().is_empty() {
        return Err(AppError::ValidationFailed(
            "GEMINI_API_KEY is not configured".to_string(),
            Some(HashMap::from([(
                "gemini_api_key".to_string(),
                "missing".to_string(),
            )])),
        ));
    }

    if audio_bytes.is_empty() {
        return Err(AppError::ValidationFailed(
            "Audio payload is empty".to_string(),
            Some(HashMap::from([("audio".to_string(), "empty".to_string())])),
        ));
    }

    let b64_audio = BASE64_STANDARD.encode(audio_bytes);
    let prompt = match lang_hint {
        Some(l) if l.starts_with("ru") => {
            "Пожалуйста, точно транскрибируйте следующую аудиозапись семейной истории на русском языке. Выведите исключительно расшифрованный текст без вступлений, таймкодов, комментариев и кавычек."
        }
        Some(l) if l.starts_with("en") => {
            "Please accurately transcribe the following spoken audio recording of a family story in English. Output only the transcribed text narrative without any preamble, timestamps, commentary, or markdown quotes."
        }
        _ => {
            "Please accurately transcribe the following spoken audio recording of a family story in its original spoken language (primarily Russian or English). Output only the transcribed text narrative without any preamble, timestamps, commentary, or markdown quotes."
        }
    };

    let effective_mime = if mime_type.is_empty() {
        "audio/webm"
    } else {
        mime_type
    };

    let request_body = json!({
        "contents": [
            {
                "role": "user",
                "parts": [
                    { "text": prompt },
                    {
                        "inlineData": {
                            "mimeType": effective_mime,
                            "data": b64_audio
                        }
                    }
                ]
            }
        ],
        "generationConfig": {
            "temperature": 0.0
        }
    });

    let endpoint =
        format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build HTTP client: {e}")))?;

    let response = client
        .post(&endpoint)
        .header("x-goog-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            AppError::Internal(format!(
                "Failed to send transcription request to Gemini: {e}"
            ))
        })?;

    let status = response.status();
    if !status.is_success() {
        let err_body = response.text().await.unwrap_or_default();
        let truncated_err: String = err_body.chars().take(200).collect();
        tracing::error!("Gemini audio transcription failed with HTTP {status}: {truncated_err}");
        return Err(AppError::ValidationFailed(
            format!("Gemini transcription failed ({status})"),
            Some(HashMap::from([(
                "audio".to_string(),
                "transcription_failed".to_string(),
            )])),
        ));
    }

    let json_val: serde_json::Value = response.json().await.map_err(|e| {
        AppError::Internal(format!(
            "Failed to parse Gemini transcription response JSON: {e}"
        ))
    })?;

    let transcript = json_val
        .get("candidates")
        .and_then(|c| c.as_array())
        .and_then(|c| c.first())
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
        .and_then(|p| p.first())
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    if transcript.is_empty() {
        return Err(AppError::ValidationFailed(
            "Gemini returned an empty transcription for this audio".to_string(),
            Some(HashMap::from([(
                "audio".to_string(),
                "empty_transcript".to_string(),
            )])),
        ));
    }

    Ok(transcript)
}

/// Extracts a structured genealogical graph (`persons`, `unions`, `parent_child_links`, `events`)
/// from a narrative text using Google Gemini generateContent with JSON mode.
pub async fn extract_story_with_gemini(
    api_key: &str,
    model: &str,
    raw_text: &str,
    speaker_context: Option<&DraftPersonDto>,
) -> Result<DraftExtractedDataDto, AppError> {
    if api_key.trim().is_empty() {
        return Err(AppError::ValidationFailed(
            "GEMINI_API_KEY is not configured".to_string(),
            Some(HashMap::from([(
                "gemini_api_key".to_string(),
                "missing".to_string(),
            )])),
        ));
    }

    let trimmed_text = raw_text.trim();
    if trimmed_text.is_empty() {
        return Err(AppError::ValidationFailed(
            "Story text must not be empty".to_string(),
            Some(HashMap::from([(
                "raw_text".to_string(),
                "required".to_string(),
            )])),
        ));
    }

    let mut prompt = String::from(
        r#"You are an expert genealogical data extraction assistant.
Analyze the following free-form family story narrative and extract all mentioned individuals, marital/partner unions, parent-child lineages, and vital life events into structured JSON.

Story text:
"""
"#,
    );
    prompt.push_str(trimmed_text);
    prompt.push_str("\n\"\"\"\n\n");

    if let Some(speaker) = speaker_context {
        prompt.push_str(&format!(
            "Known Narrator / Speaker context:
- Given name: {}
- Surname: {}
- Patronymic: {}
- Sex: {}
- Birth date: {}
- Birth place: {}
Assign the narrator person node relation_to_speaker = 'self'.

",
            speaker.given_name,
            speaker.surname,
            speaker.patronymic.as_deref().unwrap_or("none"),
            speaker.sex,
            speaker.birth_date_text.as_deref().unwrap_or("unknown"),
            speaker.birth_place.as_deref().unwrap_or("unknown"),
        ));
    }

    prompt.push_str(
r#"Strictly output ONLY a valid JSON object matching this exact schema:
{
  "persons": [
    {
      "temp_id": "draft-p-1",
      "given_name": "String (given name / имя)",
      "surname": "String (family name / фамилия)",
      "patronymic": "String or null (отчество for Russian/Slavic names, e.g. Ивановна / Сергеевич)",
      "maiden_name": "String or null (девичья фамилия if female)",
      "sex": "M | F | U",
      "relation_to_speaker": "self | father | mother | spouse | son | daughter | grandfather | grandmother | brother | sister | relative",
      "birth_date_text": "String or null (e.g. 1995-06-29, 1948, or abt 1894)",
      "birth_year": 1948,
      "birth_place": "String or null",
      "death_date_text": "String or null",
      "death_year": null,
      "death_place": "String or null",
      "is_living": true,
      "occupation": "String or null",
      "biography_notes": "String or null",
      "confidence": 90,
      "grounding_quote": "String or null (excerpt from the text mentioning this individual)"
    }
  ],
  "unions": [
    {
      "temp_id": "draft-u-1",
      "partner1_temp_id": "draft-p-1",
      "partner2_temp_id": "draft-p-2",
      "union_type": "married | unmarried | divorced | unknown",
      "marriage_date_text": "String or null",
      "marriage_place": "String or null",
      "confidence": 90
    }
  ],
  "parent_child_links": [
    {
      "parent_temp_id": "draft-p-1",
      "child_temp_id": "draft-p-3",
      "pedigree": "birth | adopted | foster"
    }
  ],
  "events": [
    {
      "temp_id": "draft-e-1",
      "person_temp_id": "draft-p-1",
      "event_type": "birth | death | marriage | residence | military | education | work | fact",
      "date_text": "String or null",
      "place_text": "String or null",
      "description": "String or null",
      "grounding_quote": "String or null"
    }
  ]
}

Key extraction guidelines:
1. First-Person Narrator: If the story is told in first person ("I", "my", "меня зовут", "мой отец", "я родился"), create a person node for the narrator and assign relation_to_speaker = 'self'.
2. Slavic Names: Separate tripartite Slavic names correctly: given_name, patronymic, surname.
3. Maiden Names: If a woman has a maiden name mentioned ("урождённая", "девичья фамилия", "née"), store it in maiden_name.
4. Living Status: If death is explicitly mentioned or birth date is more than 110 years ago, set is_living to false. Otherwise set is_living to true.
5. Kinship & Unions: If partners share children or are referred to as husband/wife/spouse/partner, create a union AND create parent_child_links for each parent-child link.
6. Return purely valid JSON with no markdown wrapping or preamble.
"#);

    let request_body = json!({
        "contents": [
            {
                "role": "user",
                "parts": [{ "text": prompt }]
            }
        ],
        "generationConfig": {
            "responseMimeType": "application/json",
            "temperature": 0.1
        }
    });

    let endpoint =
        format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build HTTP client: {e}")))?;

    let response = client
        .post(&endpoint)
        .header("x-goog-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            AppError::Internal(format!("Failed to send extraction request to Gemini: {e}"))
        })?;

    let status = response.status();
    if !status.is_success() {
        let err_body = response.text().await.unwrap_or_default();
        let truncated_err: String = err_body.chars().take(200).collect();
        tracing::error!("Gemini narrative extraction failed with HTTP {status}: {truncated_err}");
        return Err(AppError::ValidationFailed(
            format!("Gemini extraction failed ({status})"),
            Some(HashMap::from([(
                "story".to_string(),
                "extraction_failed".to_string(),
            )])),
        ));
    }

    let json_val: serde_json::Value = response.json().await.map_err(|e| {
        AppError::Internal(format!(
            "Failed to parse Gemini extraction response JSON: {e}"
        ))
    })?;

    let text_output = json_val
        .get("candidates")
        .and_then(|c| c.as_array())
        .and_then(|c| c.first())
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
        .and_then(|p| p.first())
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .trim();

    if text_output.is_empty() {
        return Err(AppError::ValidationFailed(
            "Gemini returned an empty extraction result".to_string(),
            Some(HashMap::from([(
                "story".to_string(),
                "empty_result".to_string(),
            )])),
        ));
    }

    // Clean potential markdown fences
    let cleaned_json = text_output
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let mut extracted: DraftExtractedDataDto = serde_json::from_str(cleaned_json).map_err(|e| {
        tracing::error!("Failed to parse Gemini extraction output as DraftExtractedDataDto: {e}");
        AppError::ValidationFailed(
            format!("Failed to parse structured JSON output returned by Gemini: {e}"),
            Some(HashMap::from([(
                "story".to_string(),
                "invalid_json".to_string(),
            )])),
        )
    })?;

    // Post-sanitize fields for robust downstream processing
    for (idx, p) in extracted.persons.iter_mut().enumerate() {
        if p.temp_id.trim().is_empty() {
            p.temp_id = format!("draft-p-{}", idx + 1);
        }
        let raw_sex = p.sex.trim().to_ascii_uppercase();
        p.sex = if raw_sex == "M" || raw_sex == "F" {
            raw_sex
        } else {
            "U".to_string()
        };
        if p.death_date_text.is_some() || p.death_year.is_some() || p.death_place.is_some() {
            p.is_living = false;
        }
        if p.confidence == 0 {
            p.confidence = 85;
        }
    }

    for (idx, u) in extracted.unions.iter_mut().enumerate() {
        if u.temp_id.trim().is_empty() {
            u.temp_id = format!("draft-u-{}", idx + 1);
        }
        if u.confidence == 0 {
            u.confidence = 85;
        }
    }

    for (idx, e) in extracted.events.iter_mut().enumerate() {
        if e.temp_id.trim().is_empty() {
            e.temp_id = format!("draft-e-{}", idx + 1);
        }
    }

    Ok(extracted)
}
