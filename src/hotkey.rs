use anyhow::{Context, Result, anyhow};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};

const KEYS: &[(char, Code)] = &[
    ('A', Code::KeyA),
    ('B', Code::KeyB),
    ('C', Code::KeyC),
    ('D', Code::KeyD),
    ('E', Code::KeyE),
    ('F', Code::KeyF),
    ('G', Code::KeyG),
    ('H', Code::KeyH),
    ('I', Code::KeyI),
    ('J', Code::KeyJ),
    ('K', Code::KeyK),
    ('L', Code::KeyL),
    ('M', Code::KeyM),
    ('N', Code::KeyN),
    ('O', Code::KeyO),
    ('P', Code::KeyP),
    ('Q', Code::KeyQ),
    ('R', Code::KeyR),
    ('S', Code::KeyS),
    ('T', Code::KeyT),
    ('U', Code::KeyU),
    ('V', Code::KeyV),
    ('W', Code::KeyW),
    ('X', Code::KeyX),
    ('Y', Code::KeyY),
    ('Z', Code::KeyZ),
    ('0', Code::Digit0),
    ('1', Code::Digit1),
    ('2', Code::Digit2),
    ('3', Code::Digit3),
    ('4', Code::Digit4),
    ('5', Code::Digit5),
    ('6', Code::Digit6),
    ('7', Code::Digit7),
    ('8', Code::Digit8),
    ('9', Code::Digit9),
];

pub fn parse_hotkey(configured: &str, fallback: HotKey) -> HotKey {
    match parse_hotkey_inner(configured) {
        Ok(hotkey) => hotkey,
        Err(error) => {
            eprintln!("Hotkey invalide '{configured}', fallback utilisé: {error:#}");
            fallback
        }
    }
}

fn parse_hotkey_inner(configured: &str) -> Result<HotKey> {
    let parts: Vec<&str> = configured
        .split('+')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .collect();

    if parts.is_empty() {
        return Err(anyhow!("Hotkey vide"));
    }

    let mut modifiers = Modifiers::empty();
    let mut code: Option<Code> = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "super" | "meta" | "cmd" | "command" | "win" => modifiers |= Modifiers::SUPER,
            key => {
                if code.is_some() {
                    return Err(anyhow!("Trop de touches principales"));
                }
                code = Some(parse_key_code(key)?);
            }
        }
    }

    let code = code.context("Touche principale manquante")?;
    Ok(HotKey::new(Some(modifiers), code))
}

fn parse_key_code(key: &str) -> Result<Code> {
    let upper = key.to_uppercase();
    let mut chars = upper.chars();
    if let (Some(ch), None) = (chars.next(), chars.next())
        && let Some(&(_, code)) = KEYS.iter().find(|(c, _)| *c == ch)
    {
        return Ok(code);
    }
    Err(anyhow!("Touche invalide: {key}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_code_letters() {
        assert!(matches!(parse_key_code("a"), Ok(Code::KeyA)));
        assert!(matches!(parse_key_code("Z"), Ok(Code::KeyZ)));
        assert!(matches!(parse_key_code("w"), Ok(Code::KeyW)));
    }

    #[test]
    fn test_parse_key_code_digits() {
        assert!(matches!(parse_key_code("0"), Ok(Code::Digit0)));
        assert!(matches!(parse_key_code("9"), Ok(Code::Digit9)));
    }

    #[test]
    fn test_parse_key_code_invalid() {
        assert!(parse_key_code("!").is_err());
        assert!(parse_key_code("ab").is_err());
        assert!(parse_key_code("").is_err());
    }

    #[test]
    fn test_parse_hotkey_inner_super_w() {
        let hotkey = parse_hotkey_inner("Super+W").unwrap();
        let expected = HotKey::new(Some(Modifiers::SUPER), Code::KeyW);
        assert_eq!(hotkey, expected);
    }

    #[test]
    fn test_parse_hotkey_inner_alt_a() {
        let hotkey = parse_hotkey_inner("Alt+A").unwrap();
        let expected = HotKey::new(Some(Modifiers::ALT), Code::KeyA);
        assert_eq!(hotkey, expected);
    }

    #[test]
    fn test_parse_hotkey_inner_ctrl_shift_k() {
        let hotkey = parse_hotkey_inner("Ctrl+Shift+K").unwrap();
        let expected = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyK);
        assert_eq!(hotkey, expected);
    }

    #[test]
    fn test_parse_hotkey_inner_empty() {
        assert!(parse_hotkey_inner("").is_err());
    }

    #[test]
    fn test_parse_hotkey_inner_no_key() {
        assert!(parse_hotkey_inner("Alt+Shift").is_err());
    }

    #[test]
    fn test_parse_hotkey_inner_too_many_keys() {
        assert!(parse_hotkey_inner("A+B").is_err());
    }

    #[test]
    fn test_parse_hotkey_fallback() {
        let fallback = HotKey::new(Some(Modifiers::ALT), Code::KeyA);
        let result = parse_hotkey("invalid!!!", fallback);
        assert_eq!(result, fallback);
    }
}
