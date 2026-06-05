use pinyin::{Pinyin, ToPinyin};

pub fn pinyin_aliases(value: &str) -> Vec<String> {
    let syllables = value
        .to_pinyin()
        .collect::<Option<Vec<Pinyin>>>()
        .unwrap_or_default();

    if syllables.is_empty() {
        return Vec::new();
    }

    let full = syllables
        .iter()
        .map(|pinyin| pinyin.plain())
        .collect::<String>();
    let initials = syllables
        .iter()
        .map(|pinyin| pinyin.first_letter())
        .collect::<String>();

    [full, initials]
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_chinese_to_full_pinyin_and_initials() {
        assert_eq!(pinyin_aliases("微信"), ["weixin", "wx"]);
        assert_eq!(pinyin_aliases("系统设置"), ["xitongshezhi", "xtsz"]);
    }

    #[test]
    fn ignores_non_chinese_values() {
        assert!(pinyin_aliases("WeChat").is_empty());
    }
}
