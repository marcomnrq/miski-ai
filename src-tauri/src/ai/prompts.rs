/// Build the structured summarization prompt for meeting transcripts.
///
/// Instructs the LLM to output Markdown with specific sections:
/// ## Summary, ## Participants, ## Key Topics, ## Key Points, ## Action Items
pub fn build_summary_prompt(
    transcript: &str,
    duration_minutes: u32,
    language: &str,
    notes: Option<&str>,
) -> String {
    let notes_section = match notes {
        Some(n) if !n.trim().is_empty() => {
            format!(
                "\nUser notes taken during the meeting:\n{}\n",
                n.trim()
            )
        }
        _ => String::new(),
    };

    let language_instruction = if language == "auto" {
        "Respond in the same language as the transcript.".to_string()
    } else {
        format!("Respond in {}.", language)
    };

    format!(
        r#"You are an AI meeting assistant. Analyze the following meeting transcript and produce a structured summary.

Meeting duration: {duration_minutes} minutes
{language_instruction}
{notes_section}
Transcript:
{transcript}

Format your response in Markdown with these exact sections:

## Summary
(2-3 sentence overview of what was discussed)

## Participants
(List the people or roles identified in the meeting, or "Unknown" if unclear)

## Key Topics
### Topic Name
(Brief discussion of each major topic)

## Key Points
- Key point 1
- Key point 2
- Key point 3

## Action Items
- [ ] Action item 1
- [ ] Action item 2

Important rules:
- Be concise but thorough
- Capture specific decisions, deadlines, and owners when mentioned
- Include all action items with assignees if mentioned
- Do not add information not present in the transcript
"#,
        duration_minutes = duration_minutes,
        language_instruction = language_instruction,
        notes_section = notes_section,
        transcript = transcript,
    )
}

/// Build the title generation prompt.
pub fn build_title_prompt(summary: &str, _transcript: &str, language: &str) -> String {
    let language_instruction = if language == "auto" {
        "Use the same language as the summary."
    } else {
        "Respond in English."
    };

    format!(
        r#"Generate a short, descriptive title (max 8 words) for this meeting. {language_instruction} Only return the title text, nothing else — no quotes, no punctuation at the end.

Meeting summary:
{summary}"#,
    )
}

/// Build a Q&A prompt for querying a single transcript.
pub fn build_query_prompt(transcript: &str, question: &str, language: &str) -> String {
    let language_instruction = if language == "auto" {
        "Respond in the same language as the question.".to_string()
    } else {
        format!("Respond in {}.", language)
    };

    format!(
        r#"You are an AI meeting assistant. Answer the following question based on the meeting transcript. {language_instruction}

If the answer cannot be found in the transcript, say so honestly.

Transcript:
{transcript}

Question: {question}

Answer:"#,
    )
}

/// Build a Q&A prompt for querying across multiple transcripts.
pub fn build_corpus_query_prompt(corpus: &str, question: &str, language: &str) -> String {
    let language_instruction = if language == "auto" {
        "Respond in the same language as the question.".to_string()
    } else {
        format!("Respond in {}.", language)
    };

    format!(
        r#"You are an AI meeting assistant. Answer the following question based on the collection of meeting transcripts below. {language_instruction}

If the answer cannot be found in the transcripts, say so honestly. Reference which meeting(s) the information comes from when possible.

Meeting transcripts:
{corpus}

Question: {question}

Answer:"#,
    )
}