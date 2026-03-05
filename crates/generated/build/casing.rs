use num2words::Num2Words;

pub fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            if let Ok(num) = word.parse::<i64>() {
                let spelled_out = Num2Words::new(num).to_words().unwrap_or_default();
                spelled_out
                    .split(|c: char| !c.is_alphanumeric())
                    .map(capitalize)
                    .collect::<String>()
            } else {
                capitalize(word)
            }
        })
        .collect()
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
