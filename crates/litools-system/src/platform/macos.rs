use std::{
    collections::BTreeSet,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

use plist::Value;

use crate::{DiscoveredApp, SystemAdapter, pinyin::pinyin_aliases};

#[derive(Default)]
pub struct MacosSystemAdapter;

impl SystemAdapter for MacosSystemAdapter {
    fn discover_apps(&self) -> Vec<DiscoveredApp> {
        discover_apps_from(default_application_dirs())
    }

    fn launch_app(&self, app_id: &str) -> Result<(), String> {
        launch_app(app_id)
    }

    fn open_file(&self, path: &str) -> Result<(), String> {
        Command::new("open")
            .arg(path)
            .status()
            .map_err(|error| error.to_string())?
            .success()
            .then_some(())
            .ok_or_else(|| format!("打开文件失败：{path}"))
    }
}

pub fn application_dirs() -> Vec<PathBuf> {
    default_application_dirs()
}

fn default_application_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
    ];

    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join("Applications"));
    }

    dirs
}

fn discover_apps_from(dirs: Vec<PathBuf>) -> Vec<DiscoveredApp> {
    let mut apps = Vec::new();

    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            collect_app_bundle(&path, &mut apps);

            if path.is_dir() && !is_app_bundle(&path) {
                collect_child_app_bundles(&path, &mut apps);
            }
        }
    }

    apps.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.path.cmp(&right.path))
    });
    apps.dedup_by(|left, right| left.id == right.id);
    apps
}

fn collect_child_app_bundles(dir: &Path, apps: &mut Vec<DiscoveredApp>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        collect_app_bundle(&entry.path(), apps);
    }
}

fn collect_app_bundle(path: &Path, apps: &mut Vec<DiscoveredApp>) {
    if let Some(app) = is_app_bundle(path).then(|| app_from_bundle(path)).flatten() {
        apps.push(app);
    }
}

fn is_app_bundle(path: &Path) -> bool {
    path.is_dir()
        && path
            .extension()
            .is_some_and(|extension| extension == OsStr::new("app"))
}

fn app_from_bundle(path: &Path) -> Option<DiscoveredApp> {
    let plist = read_info_plist(path);
    let localized_resources = localized_info_plist_resources(path);
    let fallback_name = plist
        .as_ref()
        .and_then(display_name_from_plist)
        .unwrap_or_else(|| bundle_file_name(path));
    let name =
        preferred_localized_name(&localized_resources).unwrap_or_else(|| fallback_name.clone());
    let id = plist
        .as_ref()
        .and_then(bundle_identifier_from_plist)
        .unwrap_or_else(|| path_id(path));
    let icon_path = plist
        .as_ref()
        .and_then(|plist| icon_path_from_plist(path, plist));
    let localized_names = localized_names(&localized_resources, &fallback_name, &name);
    let aliases = aliases_for_app(&id, &name, &localized_names, path, &localized_resources);
    let bundle_path = path.display().to_string();
    let search_text = search_text_for_app(&id, &name, &bundle_path, &localized_names, &aliases);

    Some(DiscoveredApp {
        id,
        name,
        path: bundle_path,
        icon_path,
        localized_names,
        aliases,
        search_text,
    })
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct LocalizedInfoPlist {
    locale: String,
    display_name: Option<String>,
    bundle_name: Option<String>,
    synonyms: Vec<String>,
}

fn read_info_plist(bundle_path: &Path) -> Option<Value> {
    Value::from_file(bundle_path.join("Contents/Info.plist")).ok()
}

fn localized_info_plist_resources(bundle_path: &Path) -> Vec<LocalizedInfoPlist> {
    let resources_path = bundle_path.join("Contents/Resources");
    let mut resources =
        localized_info_plist_from_loctable(&resources_path.join("InfoPlist.loctable"));
    let Ok(entries) = std::fs::read_dir(resources_path) else {
        return resources;
    };

    resources.extend(
        entries
            .flatten()
            .filter_map(|entry| localized_info_plist_from_lproj(&entry.path())),
    );
    resources
}

fn localized_info_plist_from_lproj(path: &Path) -> Option<LocalizedInfoPlist> {
    if !path.is_dir()
        || !path
            .extension()
            .is_some_and(|extension| extension == OsStr::new("lproj"))
    {
        return None;
    }

    let locale = path.file_stem()?.to_string_lossy().to_string();
    let values = parse_strings_file(&path.join("InfoPlist.strings"))?;
    Some(localized_info_plist_from_values(locale, values))
}

fn localized_info_plist_from_loctable(path: &Path) -> Vec<LocalizedInfoPlist> {
    let Some(table) = Value::from_file(path)
        .ok()
        .and_then(|value| value.into_dictionary())
    else {
        return Vec::new();
    };

    table
        .into_iter()
        .filter_map(|(locale, value)| {
            let values = value
                .into_dictionary()?
                .into_iter()
                .filter_map(|(key, value)| value.into_string().map(|value| (key, value)))
                .collect::<Vec<_>>();
            Some(localized_info_plist_from_values(locale, values))
        })
        .collect()
}

fn localized_info_plist_from_values(
    locale: String,
    values: Vec<(String, String)>,
) -> LocalizedInfoPlist {
    let display_name = localized_string_value(&values, "CFBundleDisplayName");
    let bundle_name = localized_string_value(&values, "CFBundleName");
    let synonyms = values
        .into_iter()
        .filter(|(key, _)| key.starts_with("APP_NAME_SYNONYM_"))
        .map(|(_, value)| value)
        .filter(|value| !value.trim().is_empty())
        .collect();

    LocalizedInfoPlist {
        locale,
        display_name,
        bundle_name,
        synonyms,
    }
}

fn localized_string_value(values: &[(String, String)], key: &str) -> Option<String> {
    values
        .iter()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn display_name_from_plist(plist: &Value) -> Option<String> {
    string_from_plist(plist, "CFBundleDisplayName")
        .or_else(|| string_from_plist(plist, "CFBundleName"))
}

fn preferred_localized_name(resources: &[LocalizedInfoPlist]) -> Option<String> {
    let locales = preferred_locales();
    locales
        .iter()
        .filter_map(|locale| {
            resources
                .iter()
                .find(|resource| locale_matches(&resource.locale, locale))
        })
        .find_map(localized_resource_name)
        .or_else(|| resources.iter().find_map(localized_resource_name))
}

fn localized_resource_name(resource: &LocalizedInfoPlist) -> Option<String> {
    resource
        .display_name
        .clone()
        .or_else(|| resource.bundle_name.clone())
}

fn preferred_locales() -> Vec<String> {
    let mut locales = Vec::new();
    if let Ok(languages) = std::env::var("AppleLanguages") {
        locales.extend(
            languages
                .trim_matches(|ch| matches!(ch, '(' | ')' | ' '))
                .split(',')
                .map(|language| language.trim().trim_matches('"').to_string())
                .filter(|language| !language.is_empty()),
        );
    }
    if let Ok(language) = std::env::var("LANG")
        && let Some(locale) = language.split('.').next()
    {
        locales.push(locale.replace('_', "-"));
    }
    locales.extend([
        "zh-Hans".to_string(),
        "zh-CN".to_string(),
        "zh_CN".to_string(),
        "Chinese".to_string(),
        "zh-Hant".to_string(),
    ]);
    unique_values(locales)
}

fn locale_matches(candidate: &str, preferred: &str) -> bool {
    let candidate = normalized_locale(candidate);
    let preferred = normalized_locale(preferred);
    candidate == preferred || candidate.starts_with(&format!("{preferred}-"))
}

fn normalized_locale(locale: &str) -> String {
    match locale.replace('_', "-").to_lowercase().as_str() {
        "zh-cn" | "zh-hans-cn" => "zh-hans".to_string(),
        "zh-tw" | "zh-hk" | "zh-mo" | "zh-hant-tw" | "zh-hant-hk" | "zh-hant-mo" => {
            "zh-hant".to_string()
        }
        locale => locale.to_string(),
    }
}

fn localized_names(
    resources: &[LocalizedInfoPlist],
    fallback_name: &str,
    preferred_name: &str,
) -> Vec<String> {
    unique_values(
        resources
            .iter()
            .flat_map(|resource| [resource.display_name.clone(), resource.bundle_name.clone()])
            .flatten()
            .chain([fallback_name.to_string(), preferred_name.to_string()])
            .collect(),
    )
}

fn aliases_for_app(
    id: &str,
    name: &str,
    localized_names: &[String],
    path: &Path,
    resources: &[LocalizedInfoPlist],
) -> Vec<String> {
    let mut aliases = Vec::new();
    aliases.extend(localized_names.iter().cloned());
    aliases.extend(
        resources
            .iter()
            .flat_map(|resource| resource.synonyms.clone()),
    );
    aliases.push(bundle_file_name(path));
    aliases.push(id.to_string());
    aliases.extend(acronym(name));
    for localized_name in localized_names {
        aliases.extend(acronym(localized_name));
        aliases.extend(pinyin_aliases(localized_name));
    }
    unique_values(aliases)
}

fn search_text_for_app(
    id: &str,
    name: &str,
    path: &str,
    localized_names: &[String],
    aliases: &[String],
) -> String {
    unique_values(
        [name.to_string(), id.to_string(), path.to_string()]
            .into_iter()
            .chain(localized_names.iter().cloned())
            .chain(aliases.iter().cloned())
            .collect(),
    )
    .join(" ")
}

fn acronym(value: &str) -> Option<String> {
    if value.chars().any(|ch| ch.is_ascii_whitespace())
        && value.chars().all(|ch| ch.is_ascii() || ch.is_whitespace())
    {
        let acronym = value
            .split_whitespace()
            .filter_map(|word| word.chars().find(|ch| ch.is_ascii_alphanumeric()))
            .collect::<String>()
            .to_lowercase();
        return (!acronym.is_empty()).then_some(acronym);
    }

    None
}

fn unique_values(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut unique = Vec::new();

    for value in values {
        let value = value.trim().to_string();
        if value.is_empty() {
            continue;
        }

        let key = value.to_lowercase();
        if seen.insert(key) {
            unique.push(value);
        }
    }

    unique
}

fn bundle_identifier_from_plist(plist: &Value) -> Option<String> {
    string_from_plist(plist, "CFBundleIdentifier")
}

fn parse_strings_file(path: &Path) -> Option<Vec<(String, String)>> {
    let content = read_strings_content(path)?;
    Some(parse_strings_content(&content))
}

fn read_strings_content(path: &Path) -> Option<String> {
    decode_strings_content(&std::fs::read(path).ok()?)
}

fn decode_strings_content(bytes: &[u8]) -> Option<String> {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return decode_utf16_bytes(&bytes[2..], true);
    }

    if bytes.starts_with(&[0xFE, 0xFF]) {
        return decode_utf16_bytes(&bytes[2..], false);
    }

    String::from_utf8(bytes.to_vec()).ok()
}

fn decode_utf16_bytes(bytes: &[u8], little_endian: bool) -> Option<String> {
    if bytes.len() % 2 != 0 {
        return None;
    }

    let units = bytes.chunks_exact(2).map(|chunk| {
        if little_endian {
            u16::from_le_bytes([chunk[0], chunk[1]])
        } else {
            u16::from_be_bytes([chunk[0], chunk[1]])
        }
    });

    String::from_utf16(&units.collect::<Vec<_>>()).ok()
}

fn parse_strings_content(content: &str) -> Vec<(String, String)> {
    let mut parser = StringsParser::new(content);
    let mut values = Vec::new();

    while !parser.is_finished() {
        let Some(key) = parser.next_string_token() else {
            continue;
        };
        parser.skip_whitespace_and_comments();
        if !parser.consume('=') {
            continue;
        }
        parser.skip_whitespace_and_comments();
        let Some(value) = parser.next_string_token() else {
            continue;
        };
        parser.skip_whitespace_and_comments();
        let _ = parser.consume(';');
        values.push((key, value));
    }

    values
}

struct StringsParser<'a> {
    chars: Vec<char>,
    cursor: usize,
    _source: std::marker::PhantomData<&'a str>,
}

impl<'a> StringsParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().collect(),
            cursor: 0,
            _source: std::marker::PhantomData,
        }
    }

    fn next_string_token(&mut self) -> Option<String> {
        self.skip_whitespace_and_comments();
        if self.chars.get(self.cursor) == Some(&'"') {
            return self.next_quoted_string();
        }

        self.next_unquoted_string()
    }

    fn next_quoted_string(&mut self) -> Option<String> {
        if !self.consume('"') {
            self.cursor = self.cursor.saturating_add(1);
            return None;
        }

        let mut value = String::new();
        while self.cursor < self.chars.len() {
            let ch = self.chars[self.cursor];
            self.cursor += 1;

            match ch {
                '"' => return Some(value),
                '\\' => {
                    if let Some(escaped) = self.next_escape() {
                        value.push(escaped);
                    }
                }
                _ => value.push(ch),
            }
        }

        None
    }

    fn next_unquoted_string(&mut self) -> Option<String> {
        let start = self.cursor;
        while self.chars.get(self.cursor).is_some_and(|ch| {
            !ch.is_whitespace()
                && !matches!(ch, '=' | ';' | '"')
                && !self.starts_with("//")
                && !self.starts_with("/*")
        }) {
            self.cursor += 1;
        }

        let value = self.chars[start..self.cursor]
            .iter()
            .collect::<String>()
            .trim()
            .to_string();
        if value.is_empty() {
            self.cursor = self.cursor.saturating_add(1);
            return None;
        }

        Some(value)
    }

    fn next_escape(&mut self) -> Option<char> {
        let ch = *self.chars.get(self.cursor)?;
        self.cursor += 1;
        Some(match ch {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '"' => '"',
            '\\' => '\\',
            other => other,
        })
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            while self
                .chars
                .get(self.cursor)
                .is_some_and(|ch| ch.is_whitespace())
            {
                self.cursor += 1;
            }

            if self.starts_with("//") {
                self.cursor += 2;
                while self.chars.get(self.cursor).is_some_and(|ch| *ch != '\n') {
                    self.cursor += 1;
                }
                continue;
            }

            if self.starts_with("/*") {
                self.cursor += 2;
                while self.cursor + 1 < self.chars.len() && !self.starts_with("*/") {
                    self.cursor += 1;
                }
                self.cursor = (self.cursor + 2).min(self.chars.len());
                continue;
            }

            break;
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.chars.get(self.cursor) == Some(&expected) {
            self.cursor += 1;
            return true;
        }

        false
    }

    fn starts_with(&self, expected: &str) -> bool {
        expected
            .chars()
            .enumerate()
            .all(|(offset, ch)| self.chars.get(self.cursor + offset) == Some(&ch))
    }

    fn is_finished(&self) -> bool {
        self.cursor >= self.chars.len()
    }
}

fn string_from_plist(plist: &Value, key: &str) -> Option<String> {
    plist
        .as_dictionary()?
        .get(key)?
        .as_string()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn bundle_file_name(path: &Path) -> String {
    path.file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("Unknown App")
        .to_string()
}

fn path_id(path: &Path) -> String {
    let normalized = path.display().to_string();
    format!("path:{}", normalized.replace('/', ":"))
}

fn icon_path_from_plist(bundle_path: &Path, plist: &Value) -> Option<String> {
    let icon_file = string_from_plist(plist, "CFBundleIconFile")?;
    let icon_file = if Path::new(&icon_file).extension().is_some() {
        icon_file
    } else {
        format!("{icon_file}.icns")
    };
    let path = bundle_path.join("Contents/Resources").join(icon_file);

    path.exists().then(|| path.display().to_string())
}

fn launch_app(app_id: &str) -> Result<(), String> {
    let status = if app_id.starts_with('/') || app_id.starts_with("path:") {
        let path = app_id
            .strip_prefix("path:")
            .unwrap_or(app_id)
            .replace(':', "/");
        Command::new("open").arg(path).status()
    } else {
        Command::new("open").arg("-b").arg(app_id).status()
    }
    .map_err(|error| error.to_string())?;

    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("打开应用失败：{app_id}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_file_name_removes_app_extension() {
        assert_eq!(
            bundle_file_name(Path::new("/Applications/Safari.app")),
            "Safari"
        );
    }

    #[test]
    fn path_id_is_stable_for_path() {
        assert_eq!(
            path_id(Path::new("/Applications/Example.app")),
            "path::Applications:Example.app"
        );
    }

    #[test]
    fn parses_info_plist_strings_values() {
        let values = parse_strings_content(
            r#"
            /* localized values */
            "CFBundleDisplayName" = "系统\"设置";
            "CFBundleName" = "Settings";
            // synonym
            "APP_NAME_SYNONYM_1" = "prefs";
            "#,
        );

        assert_eq!(
            values,
            [
                ("CFBundleDisplayName".to_string(), "系统\"设置".to_string()),
                ("CFBundleName".to_string(), "Settings".to_string()),
                ("APP_NAME_SYNONYM_1".to_string(), "prefs".to_string()),
            ]
        );
    }

    #[test]
    fn decodes_utf16_strings_content() {
        let content = r#""CFBundleDisplayName" = "度管家";"#;
        let le_bytes = [vec![0xFF, 0xFE], utf16_bytes(content, true)].concat();
        let be_bytes = [vec![0xFE, 0xFF], utf16_bytes(content, false)].concat();

        assert_eq!(decode_strings_content(&le_bytes), Some(content.to_string()));
        assert_eq!(decode_strings_content(&be_bytes), Some(content.to_string()));
        assert_eq!(
            parse_strings_content(&decode_strings_content(&le_bytes).expect("decode utf16")),
            [("CFBundleDisplayName".to_string(), "度管家".to_string())]
        );
    }

    #[test]
    fn parses_utf16_info_plist_strings_when_available() {
        let path = Path::new(
            "/Applications/DuGuanJia.app/Contents/Resources/zh-Hans.lproj/InfoPlist.strings",
        );
        if !path.exists() {
            return;
        }

        let values = parse_strings_file(path).expect("parse strings");

        assert_eq!(
            localized_string_value(&values, "CFBundleDisplayName"),
            Some("度管家".to_string())
        );
    }

    fn utf16_bytes(value: &str, little_endian: bool) -> Vec<u8> {
        value
            .encode_utf16()
            .flat_map(|unit| {
                if little_endian {
                    unit.to_le_bytes()
                } else {
                    unit.to_be_bytes()
                }
            })
            .collect()
    }

    #[test]
    fn locale_matching_treats_chinese_region_and_script_as_equivalent() {
        assert!(locale_matches("zh_CN", "zh-Hans"));
        assert!(locale_matches("zh-Hans", "zh-CN"));
        assert!(locale_matches("zh_TW", "zh-Hant"));
    }

    #[test]
    fn parses_system_app_info_plist_loctable_when_available() {
        let path = Path::new(
            "/System/Applications/Utilities/Activity Monitor.app/Contents/Resources/InfoPlist.loctable",
        );
        if !path.exists() {
            return;
        }

        let resources = localized_info_plist_from_loctable(path);
        let zh_name = resources
            .iter()
            .find(|resource| locale_matches(&resource.locale, "zh-Hans"))
            .and_then(localized_resource_name);

        assert_eq!(zh_name, Some("活动监视器".to_string()));
    }

    #[test]
    fn builds_aliases_from_localized_names_synonyms_acronym_and_pinyin() {
        let resources = vec![LocalizedInfoPlist {
            locale: "zh-Hans".to_string(),
            display_name: Some("系统设置".to_string()),
            bundle_name: Some("System Settings".to_string()),
            synonyms: vec!["prefs".to_string()],
        }];
        let localized_names = localized_names(&resources, "System Settings", "系统设置");
        let aliases = aliases_for_app(
            "com.apple.SystemSettings",
            "系统设置",
            &localized_names,
            Path::new("/System/Applications/System Settings.app"),
            &resources,
        );

        assert!(aliases.contains(&"系统设置".to_string()));
        assert!(aliases.contains(&"System Settings".to_string()));
        assert!(aliases.contains(&"prefs".to_string()));
        assert!(aliases.contains(&"ss".to_string()));
        assert!(aliases.contains(&"xitongshezhi".to_string()));
        assert!(aliases.contains(&"xtsz".to_string()));
    }

    #[test]
    fn generates_deduplicated_search_text() {
        let localized_names = vec!["微信".to_string(), "WeChat".to_string()];
        let aliases = vec!["weixin".to_string(), "wx".to_string(), "WeChat".to_string()];

        let search_text = search_text_for_app(
            "com.tencent.xin",
            "微信",
            "/Applications/WeChat.app",
            &localized_names,
            &aliases,
        );

        assert_eq!(
            search_text,
            "微信 com.tencent.xin /Applications/WeChat.app WeChat weixin wx"
        );
    }
}
