use position::{to_byte_pos, PosTranslator};
use std::{collections::HashMap, ops::Range, sync::Arc};

mod position;

#[derive(Debug, Clone)]
pub struct RegexError {
    pub message: String,
}

#[derive(Debug)]
pub struct Regex {
    inner: regex::Regex,
    names: Arc<Vec<Option<String>>>,
}

impl Regex {
    /// Create a new regex from a pattern.
    ///
    /// This expects normal Rust regex syntax.
    pub fn new(patter: &str) -> Result<Self, RegexError> {
        let inner = regex::Regex::new(patter).map_err(|e| RegexError {
            message: format!("Invalid regex: {}", e),
        })?;
        let names = Arc::new(
            inner
                .capture_names()
                .map(|n| n.map(|n| n.to_owned()))
                .collect(),
        );
        Ok(Self { inner, names })
    }

    pub fn pattern(&self) -> &str {
        self.inner.as_str()
    }
    pub fn groups(&self) -> usize {
        self.names.len() - 1
    }
    pub fn groupindex(&self) -> HashMap<String, usize> {
        self.names
            .iter()
            .enumerate()
            .filter_map(|(i, n)| n.as_ref().map(|n| (n.clone(), i)))
            .collect()
    }

    pub fn search(&self, text: &str, pos: usize) -> Option<RegexMatch> {
        let mut translator = PosTranslator::new(text);
        self.inner
            .captures_at(text, to_byte_pos(text, pos))
            .map(|m| RegexMatch::from_captures(&mut translator, m, self.names.clone()))
    }

    pub fn findall<'a>(&'a self, text: &'a str) -> impl Iterator<Item = RegexMatch> + 'a {
        let mut translator = PosTranslator::new(text);
        self.inner
            .captures_iter(text)
            .map(move |m| RegexMatch::from_captures(&mut translator, m, self.names.clone()))
    }

    pub fn split(&self, text: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut last = 0;

        for m in self.inner.captures_iter(text) {
            let start = m.get(0).unwrap().start();
            let end = m.get(0).unwrap().end();
            if start > last {
                result.push(text[last..start].to_owned());
            }
            last = end;

            for g in m.iter().skip(1).flatten() {
                result.push(g.as_str().to_owned());
            }
        }

        if last < text.len() {
            result.push(text[last..].to_owned());
        }

        result
    }

    pub fn split_without_captures(&self, text: &str) -> Vec<String> {
        self.inner.split(text).map(|s| s.to_owned()).collect()
    }
}

#[derive(Debug)]
pub struct RegexMatch {
    groups: Vec<Option<Range<usize>>>,
    names: Arc<Vec<Option<String>>>,
}

impl RegexMatch {
    fn first(&self) -> &Range<usize> {
        self.groups[0]
            .as_ref()
            .expect("first to always to be present")
    }

    fn from_captures<'t>(
        translator: &mut PosTranslator<'t>,
        captures: regex::Captures<'t>,
        names: Arc<Vec<Option<String>>>,
    ) -> Self {
        Self {
            groups: captures
                .iter()
                .map(|g| {
                    g.map(|g| Range {
                        start: translator.get_char_pos(g.start()),
                        end: translator.get_char_pos(g.end()),
                    })
                })
                .collect(),
            names,
        }
    }
    pub fn start(&self) -> usize {
        self.first().start
    }
    pub fn end(&self) -> usize {
        self.first().end
    }

    pub fn get(&self, index: usize) -> Option<Range<usize>> {
        self.groups.get(index).cloned().flatten()
    }

    pub fn get_by_name(&self, name: &str) -> Option<Range<usize>> {
        self.names
            .iter()
            .position(|n| n.as_ref().map(|n| n.as_str()) == Some(name))
            .and_then(|i| self.groups.get(i).cloned().flatten())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_props() {
        assert_eq!(Regex::new(r"foo").unwrap().pattern(), "foo");
        assert_eq!(Regex::new(r"foo").unwrap().groups(), 0);
        assert_eq!(Regex::new(r"(foo)").unwrap().pattern(), "(foo)");
        assert_eq!(Regex::new(r"(foo)").unwrap().groups(), 1);
    }

    #[test]
    fn regex_search() {
        let re = Regex::new(r"\b(fo+)").unwrap();

        let m: RegexMatch = re.search("foo", 0).unwrap();
        assert_eq!(m.start(), 0);
        assert_eq!(m.end(), 3);
        assert_eq!(m.get(0), Some(0..3));
    }

    #[test]
    fn regex_search_unicode() {
        let re = Regex::new(r"\B.").unwrap();

        let m: RegexMatch = re.search("äöü", 0).unwrap();
        assert_eq!(m.start(), 1);
        assert_eq!(m.end(), 2);
        assert_eq!(m.get(0), Some(1..2));
    }
}
