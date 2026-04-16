//! Spell checker implementation with dictionary lookup and suggestion generation.

use std::collections::HashSet;
use crate::aff::AffixRules;

#[derive(Debug)]
pub struct SpellChecker {
    pub aff: AffixRules,
    pub words: HashSet<String>,
    pub user_words: HashSet<String>,
}

impl SpellChecker {
    /// Create from Hunspell .aff and .dic file contents
    pub fn from_files(aff_content: &str, dic_content: &str) -> Self {
        let aff = AffixRules::parse(aff_content);
        let words = parse_dic(dic_content);
        SpellChecker { aff, words, user_words: HashSet::new() }
    }

    /// Create a minimal English checker with common words built in
    pub fn new_minimal_english() -> Self {
        let mut words = HashSet::new();
        // A representative set of common English words for Phase 1
        let common_words = include_str!("../words/en_common.txt");
        for word in common_words.lines() {
            let w = word.trim().to_lowercase();
            if !w.is_empty() && !w.starts_with('#') {
                words.insert(w);
            }
        }
        SpellChecker {
            aff: AffixRules::default(),
            words,
            user_words: HashSet::new(),
        }
    }

    /// Check if a word is spelled correctly
    pub fn check(&self, word: &str) -> bool {
        let lower = word.to_lowercase();

        // Direct lookup
        if self.words.contains(&lower) || self.user_words.contains(&lower) {
            return true;
        }

        // Try suffix stripping
        self.check_with_affixes(&lower)
    }

    /// Check with affix stripping
    fn check_with_affixes(&self, word: &str) -> bool {
        // Try stripping common English suffixes
        let suffixes = &[
            "ing", "ed", "er", "est", "ly", "ness", "tion", "ation", "sion",
            "ment", "ful", "less", "able", "ible", "al", "ous", "ive", "ary",
            "ery", "ory", "cy", "ity", "ty", "ry", "s", "es", "ies",
        ];

        for suffix in suffixes {
            if word.ends_with(suffix) && word.len() > suffix.len() + 2 {
                let stem = &word[..word.len() - suffix.len()];
                if self.words.contains(stem) {
                    return true;
                }
                // Double-letter removal (running → run)
                if stem.len() > 1 {
                    let chars: Vec<char> = stem.chars().collect();
                    if chars.len() >= 2 && chars[chars.len()-1] == chars[chars.len()-2] {
                        let dedouble: String = chars[..chars.len()-1].iter().collect();
                        if self.words.contains(&dedouble) {
                            return true;
                        }
                    }
                }
                // e-restoration (hoping → hope)
                let with_e = format!("{}e", stem);
                if self.words.contains(&with_e) {
                    return true;
                }
                // y→i restoration (happiest → happy)
                if suffix.starts_with('i') || *suffix == "ies" {
                    let with_y = format!("{}y", stem.trim_end_matches('i'));
                    if self.words.contains(&with_y) {
                        return true;
                    }
                }
            }
        }

        // Try stripping common prefixes
        let prefixes = &["un", "re", "dis", "over", "under", "out", "pre", "mis"];
        for prefix in prefixes {
            if word.starts_with(prefix) && word.len() > prefix.len() + 2 {
                let stem = &word[prefix.len()..];
                if self.check(stem) { return true; }
            }
        }

        false
    }

    /// Generate spelling suggestions for a misspelled word
    pub fn suggest(&self, word: &str) -> Vec<String> {
        let lower = word.to_lowercase();
        let mut candidates: Vec<(usize, String)> = vec![];

        // First try replacement table
        for (from, to) in &self.aff.replacements {
            if lower.contains(from.as_str()) {
                let candidate = lower.replace(from.as_str(), to.as_str());
                if self.check(&candidate) {
                    candidates.push((0, candidate));
                }
            }
        }

        // Try all words, sorted by edit distance
        let max_distance = if lower.len() <= 4 { 1 } else if lower.len() <= 8 { 2 } else { 3 };

        for dict_word in &self.words {
            let dist = levenshtein(&lower, dict_word);
            if dist <= max_distance {
                candidates.push((dist, dict_word.clone()));
            }
        }

        // Also try transpositions, deletions, insertions with try_chars
        let try_chars: Vec<char> = if self.aff.try_chars.is_empty() {
            "etaoinshrdlcumwfgypbvkjxqz".chars().collect()
        } else {
            self.aff.try_chars.chars().collect()
        };

        // Transpositions
        let chars: Vec<char> = lower.chars().collect();
        for i in 0..chars.len().saturating_sub(1) {
            let mut swapped = chars.clone();
            swapped.swap(i, i + 1);
            let candidate: String = swapped.iter().collect();
            if self.check(&candidate) && !candidates.iter().any(|(_, w)| w == &candidate) {
                candidates.push((1, candidate));
            }
        }

        // Single char substitution
        for i in 0..chars.len() {
            for &c in &try_chars {
                if c != chars[i] {
                    let mut variant = chars.clone();
                    variant[i] = c;
                    let candidate: String = variant.iter().collect();
                    if self.check(&candidate) && !candidates.iter().any(|(_, w)| w == &candidate) {
                        candidates.push((1, candidate));
                    }
                }
            }
        }

        // Sort by distance, then alphabetically
        candidates.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        candidates.dedup_by(|a, b| a.1 == b.1);

        // Return top 10 suggestions, preserving original capitalization style
        let capitalize = word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
        candidates.iter()
            .take(10)
            .map(|(_, w)| if capitalize { capitalize_first(w) } else { w.clone() })
            .collect()
    }

    /// Add a word to the user dictionary
    pub fn add_word(&mut self, word: &str) {
        self.user_words.insert(word.to_lowercase());
    }

    /// Process a text string and return misspelled word positions
    pub fn check_text(&self, text: &str) -> Vec<SpellResult> {
        use unicode_segmentation::UnicodeSegmentation;
        let mut results = vec![];
        let mut pos = 0;

        for word_str in text.unicode_words() {
            let byte_pos = text[pos..].find(word_str).unwrap_or(0) + pos;

            // Skip numbers, URLs, emails
            if word_str.chars().any(|c| c.is_ascii_digit())
                || word_str.contains('@')
                || word_str.contains("://")
            {
                pos = byte_pos + word_str.len();
                continue;
            }

            // Strip punctuation
            let clean: String = word_str.chars().filter(|c| c.is_alphabetic()).collect();
            if clean.len() < 2 { pos = byte_pos + word_str.len(); continue; }

            if !self.check(&clean) {
                let suggestions = self.suggest(&clean);
                results.push(SpellResult {
                    word: word_str.to_string(),
                    byte_start: byte_pos,
                    byte_end: byte_pos + word_str.len(),
                    suggestions,
                });
            }
            pos = byte_pos + word_str.len();
        }

        results
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpellResult {
    pub word: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub suggestions: Vec<String>,
}

fn parse_dic(content: &str) -> HashSet<String> {
    let mut words = HashSet::new();
    for line in content.lines().skip(1) { // first line is count
        let word = line.split('/').next().unwrap_or(line).trim().to_lowercase();
        if !word.is_empty() {
            words.insert(word);
        }
    }
    words
}

/// Levenshtein edit distance
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    // Early exit
    if m == 0 { return n; }
    if n == 0 { return m; }
    if (m as isize - n as isize).unsigned_abs() > 5 { return usize::MAX; } // too different

    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m { dp[i][0] = i; }
    for j in 0..=n { dp[0][j] = j; }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i-1] == b_chars[j-1] { 0 } else { 1 };
            dp[i][j] = (dp[i-1][j] + 1)
                .min(dp[i][j-1] + 1)
                .min(dp[i-1][j-1] + cost);
        }
    }

    dp[m][n]
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("cat", "cat"), 0);
        assert_eq!(levenshtein("cat", "bat"), 1);
        assert_eq!(levenshtein("cat", "cats"), 1);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn test_spell_check_basic() {
        let checker = SpellChecker::new_minimal_english();
        assert!(checker.check("hello"));
        assert!(checker.check("world"));
        assert!(!checker.check("helo")); // misspelled
    }
}
