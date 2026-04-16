//! Affix rule parser for Hunspell .aff files.

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct AffixRules {
    pub try_chars: String,
    pub prefixes: HashMap<String, Vec<AffixRule>>,
    pub suffixes: HashMap<String, Vec<AffixRule>>,
    pub replacements: Vec<(String, String)>,
    pub char_maps: Vec<(String, String)>,
    pub compound_rules: Vec<String>,
    pub min_word_len: usize,
    pub lang: String,
}

#[derive(Debug, Clone)]
pub struct AffixRule {
    pub flag: String,
    pub cross_product: bool,
    pub strip: String,
    pub affix: String,
    pub condition: String,
}

impl AffixRules {
    pub fn parse(aff_content: &str) -> Self {
        let mut rules = AffixRules {
            min_word_len: 3,
            ..Default::default()
        };

        let mut lines = aff_content.lines().peekable();
        while let Some(line) = lines.next() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }

            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() < 2 { continue; }

            match parts[0] {
                "TRY" => rules.try_chars = parts[1].to_string(),
                "LANG" => rules.lang = parts[1].to_string(),
                "WORDCHARS" => {} // ignored for now
                "REP" => {
                    if let Some((from, to)) = parts[1].split_once(' ') {
                        rules.replacements.push((from.to_string(), to.to_string()));
                    }
                }
                "MAP" => {
                    if let Some((a, b)) = parts[1].split_once(' ') {
                        rules.char_maps.push((a.to_string(), b.to_string()));
                    }
                }
                "SFX" | "PFX" => {
                    let is_suffix = parts[0] == "SFX";
                    let fields: Vec<&str> = parts[1].split_whitespace().collect();
                    if fields.len() < 2 { continue; }
                    let flag = fields[0].to_string();
                    let cross = fields.get(1) == Some(&"Y");

                    // Read subsequent SFX/PFX rule lines
                    while let Some(next_line) = lines.peek() {
                        let nl = next_line.trim();
                        if nl.starts_with(if is_suffix { "SFX" } else { "PFX" }) {
                            let nfields: Vec<&str> = nl.split_whitespace().collect();
                            if nfields.len() >= 4 && nfields[1] == flag {
                                lines.next(); // consume
                                let rule = AffixRule {
                                    flag: flag.clone(),
                                    cross_product: cross,
                                    strip: if nfields[2] == "0" { String::new() } else { nfields[2].to_string() },
                                    affix: if nfields[3] == "0" { String::new() } else { nfields[3].to_string() },
                                    condition: nfields.get(4).copied().unwrap_or(".").to_string(),
                                };
                                if is_suffix {
                                    rules.suffixes.entry(flag.clone()).or_default().push(rule);
                                } else {
                                    rules.prefixes.entry(flag.clone()).or_default().push(rule);
                                }
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                _ => {}
            }
        }

        rules
    }
}
