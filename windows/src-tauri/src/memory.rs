// The chat's memory of the user: what they wrote about themselves, or pasted
// from ChatGPT (Settings → Personalisation → Memory), kept in memory.txt in
// Coucou's local folder and added to the system prompt of every provider. It
// stays on this computer, except that each chat message sends it to the
// provider that answers, like the rest of the conversation.

use std::path::PathBuf;

/// More than this would crowd out the conversation.
pub const MAX_CHARS: usize = 20_000;

fn path() -> PathBuf {
    crate::settings::local_dir().join("memory.txt")
}

/// The memory, or "" when there is none.
pub fn read() -> String {
    std::fs::read_to_string(path()).unwrap_or_default()
}

/// `text` trimmed and cut to MAX_CHARS.
pub fn clean(text: &str) -> String {
    text.trim().chars().take(MAX_CHARS).collect()
}

/// The system prompt's paragraph about the user, or "" when nothing is known.
pub fn prompt_block(memory: &str) -> String {
    let memory = memory.trim();
    if memory.is_empty() {
        return String::new();
    }
    format!(
        "\n\nWhat you know about the user (their own notes, some carried over from ChatGPT's memory). \
Use it to answer as someone who knows them, without reciting it:\n<memory>\n{memory}\n</memory>"
    )
}

#[tauri::command]
pub fn memory_get() -> String {
    read()
}

#[tauri::command]
pub fn memory_set(text: String) -> Result<usize, String> {
    let text = clean(&text);
    let dir = crate::settings::local_dir();
    crate::platform::ensure_private_dir(&dir).map_err(|e| e.to_string())?;
    if text.is_empty() {
        let _ = std::fs::remove_file(path());
    } else {
        std::fs::write(path(), &text).map_err(|e| e.to_string())?;
    }
    Ok(text.chars().count())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_block_is_empty_without_memory_and_wraps_it_otherwise() {
        assert_eq!(prompt_block("  \n"), "");
        let b = prompt_block("Ismim Umidjon. Toshkentda yashayman.");
        assert!(b.contains("<memory>\nIsmim Umidjon. Toshkentda yashayman.\n</memory>"));
    }

    #[test]
    fn cleaning_trims_and_caps() {
        assert_eq!(clean("  salom \n"), "salom");
        assert_eq!(clean(&"a".repeat(MAX_CHARS + 50)).chars().count(), MAX_CHARS);
    }
}
