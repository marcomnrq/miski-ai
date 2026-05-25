use crate::models::Meeting;

/// Export a meeting as a Markdown string with YAML frontmatter.
///
/// Output format:
/// ```markdown
/// ---
/// title: "Meeting Title"
/// date: "2026-05-25T17:30:00Z"
/// duration_seconds: 1800
/// language: "en"
/// is_diarised: true
/// ---
///
/// ## Summary
/// (summary content)
///
/// ## Transcript
/// (diarised or plain transcript)
///
/// ## User Notes
/// (notes)
/// ```
pub fn export_meeting_markdown(meeting: &Meeting) -> String {
    let mut md = String::new();

    // YAML frontmatter
    md.push_str("---\n");
    md.push_str(&format!("title: \"{}\"\n", escape_yaml(&meeting.title)));
    md.push_str(&format!("date: \"{}\"\n", meeting.created_at));
    if let Some(dur) = meeting.duration_seconds {
        md.push_str(&format!("duration_seconds: {}\n", dur as i64));
    }
    md.push_str(&format!("language: \"{}\"\n", meeting.language));
    md.push_str(&format!("is_diarised: {}\n", meeting.is_diarised));
    if let Some(ref model) = meeting.whisper_model {
        md.push_str(&format!("whisper_model: \"{}\"\n", model));
    }
    if let Some(ref model) = meeting.ollama_model {
        md.push_str(&format!("ollama_model: \"{}\"\n", model));
    }
    md.push_str("---\n\n");

    // Summary
    if let Some(ref summary) = meeting.summary_markdown {
        md.push_str(summary);
        md.push_str("\n\n");
    }

    // Transcript
    let transcript = meeting.diarised_text.as_ref()
        .or(meeting.transcript_text.as_ref());

    if let Some(text) = transcript {
        md.push_str("## Transcript\n\n");
        md.push_str(text);
        md.push_str("\n\n");
    }

    // User Notes
    if let Some(ref notes) = meeting.notes {
        if !notes.is_empty() {
            md.push_str("## User Notes\n\n");
            md.push_str(notes);
            md.push_str("\n");
        }
    }

    md
}

fn escape_yaml(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', " ")
}