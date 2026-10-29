// Mochi speaks Uzbek: an offline voice on this computer. The model is Meta's
// MMS text-to-speech for Uzbek in Latin script (VITS, CC BY-NC 4.0), turned
// into ONNX for sherpa-onnx by the build (.github/workflows/build-windows.yml)
// and shipped next to coucou.exe in voices/uz/. Nothing is sent anywhere.
//
// The webview asks for a sentence (`uz_say`) and gets a WAV back; it plays it
// through its own robot effect (src/core/uzvoice.ts).

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use sherpa_onnx::{GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsModelConfig, OfflineTtsVitsModelConfig};

/// voices/uz next to coucou.exe (where the installer puts it), else the source
/// tree's copy for `tauri dev`.
fn model_dir() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.to_path_buf())) {
        candidates.push(dir.join("voices").join("uz"));
        // However the bundler lays the folder out.
        candidates.push(dir.join("voices"));
        candidates.push(dir.join("uz"));
    }
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("voices").join("uz"));
    candidates.into_iter().find(|d| d.join("model.onnx").is_file() && d.join("tokens.txt").is_file())
}

struct Engine {
    tts: OfflineTts,
    /// Every character the model knows (tokens.txt).
    chars: HashSet<char>,
}

fn engine() -> &'static Mutex<Option<Engine>> {
    static E: OnceLock<Mutex<Option<Engine>>> = OnceLock::new();
    E.get_or_init(|| Mutex::new(None))
}

/// The characters of tokens.txt: one "<char> <id>" a line; the space is " <id>".
pub fn token_chars(tokens: &str) -> HashSet<char> {
    let mut out = HashSet::new();
    for line in tokens.lines() {
        let Some((token, id)) = line.rsplit_once(' ') else { continue };
        if id.trim().parse::<u32>().is_err() {
            continue;
        }
        let mut cs = token.chars();
        match (cs.next(), cs.next()) {
            (Some(c), None) => {
                out.insert(c);
            }
            (None, _) => {
                out.insert(' ');
            }
            _ => {}
        }
    }
    out
}

fn load() -> Result<Engine, String> {
    let dir = model_dir().ok_or("O‘zbekcha ovoz modeli topilmadi")?;
    let tokens = dir.join("tokens.txt");
    let chars = token_chars(&std::fs::read_to_string(&tokens).map_err(|e| e.to_string())?);
    let config = OfflineTtsConfig {
        model: OfflineTtsModelConfig {
            vits: OfflineTtsVitsModelConfig {
                model: Some(dir.join("model.onnx").to_string_lossy().into_owned()),
                tokens: Some(tokens.to_string_lossy().into_owned()),
                noise_scale: 0.6,
                noise_scale_w: 0.7,
                length_scale: 1.0,
                ..Default::default()
            },
            num_threads: std::thread::available_parallelism().map_or(2, |n| n.get().clamp(1, 4) as i32),
            ..Default::default()
        },
        max_num_sentences: 1,
        ..Default::default()
    };
    let tts = OfflineTts::create(&config).ok_or("O‘zbekcha ovoz modelini yuklab bo‘lmadi")?;
    Ok(Engine { tts, chars })
}

// ── Text ──────────────────────────────────────────────────────────────────────

const ONES: [&str; 10] = ["nol", "bir", "ikki", "uch", "to‘rt", "besh", "olti", "yetti", "sakkiz", "to‘qqiz"];
const TENS: [&str; 10] = ["", "o‘n", "yigirma", "o‘ttiz", "qirq", "ellik", "oltmish", "yetmish", "sakson", "to‘qson"];

/// 2026 → "ikki ming yigirma olti".
pub fn number_words(n: u64) -> String {
    fn below_thousand(n: u64, out: &mut Vec<String>) {
        let (h, rest) = (n / 100, n % 100);
        if h > 0 {
            if h > 1 {
                out.push(ONES[h as usize].into());
            }
            out.push("yuz".into());
        }
        let (t, o) = (rest / 10, rest % 10);
        if t > 0 {
            out.push(TENS[t as usize].into());
        }
        if o > 0 {
            out.push(ONES[o as usize].into());
        }
    }
    if n == 0 {
        return ONES[0].into();
    }
    let mut out = Vec::new();
    let scales = [(1_000_000_000_000u64, "trillion"), (1_000_000_000, "milliard"), (1_000_000, "million"), (1_000, "ming")];
    let mut rest = n;
    for (size, name) in scales {
        if rest >= size {
            let k = rest / size;
            // "ming", not "bir ming".
            if !(k == 1 && size == 1_000) {
                below_thousand(k % 1000, &mut out);
            }
            out.push(name.into());
            rest %= size;
        }
    }
    below_thousand(rest, &mut out);
    out.join(" ")
}

/// What the model can read: lower case, numbers in words, Uzbek's ‘ and ’
/// written the way the model learned them, and anything it has no letter for
/// turned into a pause.
pub fn normalize(text: &str, chars: &HashSet<char>) -> String {
    // The model's own apostrophe, whichever of the usual ones it has.
    let apostrophe = ['\'', 'ʻ', '‘', '’', 'ʼ', '`'].into_iter().find(|c| chars.contains(c));
    let mut spelled = String::new();
    let mut digits = String::new();
    let flush = |digits: &mut String, out: &mut String| {
        if digits.is_empty() {
            return;
        }
        let words = digits.parse::<u64>().map(number_words).unwrap_or_else(|_| {
            digits.chars().filter_map(|d| d.to_digit(10)).map(|d| ONES[d as usize]).collect::<Vec<_>>().join(" ")
        });
        out.push(' ');
        out.push_str(&words);
        out.push(' ');
        digits.clear();
    };
    for c in text.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
            continue;
        }
        flush(&mut digits, &mut spelled);
        match c {
            '%' => spelled.push_str(" foiz "),
            '+' => spelled.push_str(" plyus "),
            '&' => spelled.push_str(" va "),
            '@' => spelled.push_str(" kuchukcha "),
            _ => spelled.push(c),
        }
    }
    flush(&mut digits, &mut spelled);

    let mut out = String::new();
    for c in spelled.to_lowercase().chars() {
        let c = match c {
            'ʻ' | '‘' | '’' | 'ʼ' | '`' | '\'' => match apostrophe {
                Some(a) => a,
                None => continue,
            },
            'ё' => 'e',
            _ => c,
        };
        if chars.contains(&c) && c != ' ' {
            out.push(c);
        } else if c.is_alphanumeric() {
            // A letter the model lacks (é, ö…): its plain Latin letter when it has one.
            let plain = match c {
                'á' | 'à' | 'â' | 'ä' => 'a',
                'é' | 'è' | 'ê' | 'ë' => 'e',
                'í' | 'ì' | 'î' | 'ï' => 'i',
                'ó' | 'ò' | 'ô' | 'ö' => 'o',
                'ú' | 'ù' | 'û' | 'ü' => 'u',
                'ç' => 'c',
                'ş' => 's',
                'ğ' => 'g',
                'w' => 'v',
                _ => ' ',
            };
            out.push(if chars.contains(&plain) { plain } else { ' ' });
        } else if !out.ends_with(' ') {
            out.push(' ');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ── Audio ─────────────────────────────────────────────────────────────────────

/// 16-bit mono WAV.
pub fn wav(samples: &[f32], rate: u32) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
    }
    out
}

fn say(text: &str) -> Result<Vec<u8>, String> {
    let mut slot = engine().lock().unwrap_or_else(|e| e.into_inner());
    if slot.is_none() {
        *slot = Some(load()?);
    }
    let engine = slot.as_ref().unwrap();
    let clean = normalize(text, &engine.chars);
    if clean.is_empty() {
        return Err("Aytadigan so‘z yo‘q".into());
    }
    let config = GenerationConfig { speed: 1.05, ..Default::default() };
    let audio = engine
        .tts
        .generate_with_config(&clean, &config, None::<fn(&[f32], f32) -> bool>)
        .ok_or("Ovozni yaratib bo‘lmadi")?;
    Ok(wav(audio.samples(), audio.sample_rate().max(1) as u32))
}

/// Whether the Uzbek voice is installed with this copy.
#[tauri::command]
pub fn uz_voice_ready() -> bool {
    model_dir().is_some()
}

/// `text` said by the Uzbek voice, as a WAV.
#[tauri::command]
pub async fn uz_say(text: String) -> Result<tauri::ipc::Response, String> {
    let text: String = text.chars().take(400).collect();
    tauri::async_runtime::spawn_blocking(move || say(&text))
        .await
        .map_err(|e| e.to_string())?
        .map(tauri::ipc::Response::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uzbek_chars() -> HashSet<char> {
        token_chars("  0\n' 1\na 2\nb 3\nc 4\nd 5\ne 6\nf 7\ng 8\nh 9\ni 10\nj 11\nk 12\nl 13\nm 14\nn 15\no 16\np 17\nq 18\nr 19\ns 20\nt 21\nu 22\nv 23\nx 24\ny 25\nz 26\n, 27\n. 28\n? 29\n! 30\n")
    }

    #[test]
    fn reads_tokens_with_the_space() {
        let chars = uzbek_chars();
        assert!(chars.contains(&' '));
        assert!(chars.contains(&'\''));
        assert!(chars.contains(&'q'));
        assert!(!chars.contains(&'w'));
    }

    #[test]
    fn says_numbers_in_uzbek() {
        assert_eq!(number_words(0), "nol");
        assert_eq!(number_words(7), "yetti");
        assert_eq!(number_words(15), "o‘n besh");
        assert_eq!(number_words(100), "yuz");
        assert_eq!(number_words(345), "uch yuz qirq besh");
        assert_eq!(number_words(1000), "ming");
        assert_eq!(number_words(2026), "ikki ming yigirma olti");
        assert_eq!(number_words(1_500_000), "bir million besh yuz ming");
    }

    #[test]
    fn writes_what_the_model_can_read() {
        let chars = uzbek_chars();
        assert_eq!(normalize("Salom! Qanday yordam beray?", &chars), "salom! qanday yordam beray?");
        assert_eq!(normalize("Bo‘ldi, o‘g‘il ma’no", &chars), "bo'ldi, o'g'il ma'no");
        assert_eq!(normalize("3 ta fayl", &chars), "uch ta fayl");
        assert_eq!(normalize("Claude Code — ishni tugatdi", &chars), "claude code ishni tugatdi");
        assert_eq!(normalize("50%", &chars), "ellik foiz");
        assert_eq!(normalize("Café", &chars), "cafe");
        assert_eq!(normalize("✓ ~/.claude", &chars), ".claude");
    }

    #[test]
    fn makes_a_wav() {
        let bytes = wav(&[0.0, 1.0, -1.0], 16000);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(bytes.len(), 44 + 6);
        assert_eq!(&bytes[44..46], &0i16.to_le_bytes());
        assert_eq!(&bytes[46..48], &32767i16.to_le_bytes());
    }
}
