use essential_types::{
    convert::{bool_from_word, bytes_from_word},
    Word,
};

pub fn parse_type(words: &[Word], ty: &str) -> String {
    let ty = ty.trim();
    let force_hex = ty.ends_with("HEX");
    let ty = ty.trim_end_matches("HEX").trim();
    if ty.starts_with(|a: char| a.is_ascii_digit()) {
        let Some(pos) = ty
            .split(' ')
            .next()
            .and_then(|pos| pos.parse::<usize>().ok())
        else {
            return String::new();
        };
        let ty = ty.trim_start_matches(|a: char| a.is_ascii_digit()).trim();
        if pos >= words.len() {
            return String::new();
        }
        parse_ty(&words[pos..], ty, force_hex)
    } else {
        parse_ty(words, ty, force_hex)
    }
}

fn parse_ty(words: &[Word], ty: &str, force_hex: bool) -> String {
    let ty = ty.trim();
    if ty.starts_with(|a: char| a.is_ascii_alphabetic()) && ty.ends_with(']') {
        let mut iter = ty.trim_end_matches(']').split('[');
        let Some(ty) = iter.next() else {
            return String::new();
        };
        let Some(len) = iter.next().and_then(|len| len.parse::<usize>().ok()) else {
            return String::new();
        };
        let size = get_size(ty);
        let total_size = size * len;
        if words.len() < total_size {
            return String::new();
        }
        let out = words
            .chunks_exact(size)
            .take(len)
            .map(|words| parse_primitive(words, ty, force_hex))
            .collect::<Vec<_>>()
            .join(", ");
        let out = out.trim().trim_end_matches(',');
        format!("[{}]", out)
    } else if ty.starts_with('{') && ty.ends_with('}') {
        let tys: Vec<&str> = ty
            .trim_start_matches('{')
            .trim_end_matches('}')
            .split(',')
            .collect();
        if words.len() < get_total_size(&tys) {
            return String::new();
        }
        let mut start = 0;
        let out = tys
            .iter()
            .map(|ty| {
                let end = start + get_size(ty);
                let o = parse_primitive(&words[start..end], ty, force_hex);
                start = end;
                o
            })
            .collect::<Vec<_>>()
            .join(", ");
        let out = out.trim().trim_end_matches(',');
        format!("{{ {} }}", out)
    } else {
        parse_primitive(words, ty, force_hex)
    }
}

fn get_total_size(tys: &[&str]) -> usize {
    tys.iter().map(|ty| get_size(ty)).sum()
}

fn get_size(ty: &str) -> usize {
    match ty.trim() {
        "int" => 1,
        "bool" => 1,
        "b256" => 4,
        _ => 0,
    }
}

fn parse_primitive(words: &[Word], ty: &str, force_hex: bool) -> String {
    match ty.trim() {
        "int" => {
            let Some(word) = words.first() else {
                return String::new();
            };
            if force_hex {
                hex::encode_upper(bytes_from_word(*word))
            } else {
                format!("{}", word)
            }
        }
        "bool" => {
            let Some(word) = words.first() else {
                return String::new();
            };
            if force_hex {
                hex::encode_upper(bytes_from_word(*word))
            } else {
                bool_from_word(*word)
                    .map(|b| b.to_string())
                    .unwrap_or_default()
            }
        }
        "b256" => {
            if words.len() < 4 {
                return String::new();
            }
            let words = &words[..4];
            let bytes: Vec<u8> = words.iter().flat_map(|w| bytes_from_word(*w)).collect();
            hex::encode_upper(bytes)
        }
        _ => String::new(),
    }
}
