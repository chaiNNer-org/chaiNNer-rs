use pyo3::{exceptions::PyValueError, prelude::*};
use regex::Regex;
use std::{collections::HashMap, sync::Arc};

fn to_byte_pos(text: &str, pos: usize) -> usize {
    if pos == 0 {
        return 0;
    }
    if pos > text.len() {
        // the position is out of bounds anyway
        return pos;
    }

    if let Some(char_pos) = text.char_indices().nth(pos).map(|(i, _)| i) {
        char_pos
    } else {
        // the position is out of bounds
        pos + text.chars().count() - text.len()
    }
}

/// A helper struct for translating byte positions to Unicode character positions.
struct PosTranslator<'a> {
    text: &'a str,
    known: Vec<(usize, usize)>,
}
impl<'a> PosTranslator<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            known: vec![],
        }
    }

    pub fn get_char_pos(&mut self, byte_pos: usize) -> usize {
        let mut start_offset = (0, 0);
        for k in self.known.iter().rev() {
            if k.0 <= byte_pos {
                start_offset = *k;
                break;
            }
        }

        if start_offset.0 == byte_pos {
            return start_offset.1;
        }

        let text = &self.text[start_offset.0..];

        let mut char_pos = start_offset.1;
        for (i, _) in text.char_indices() {
            if i + start_offset.0 >= byte_pos {
                break;
            }
            char_pos += 1;
        }

        if self.known.last().map_or(true, |l| l.0 < byte_pos) {
            // keep the array sorted
            self.known.push((byte_pos, char_pos));
        }

        char_pos
    }
}

#[pyclass(frozen)]
pub struct RustRegex {
    inner: Regex,
    names: Arc<Vec<Option<String>>>,
}

#[pymethods]
impl RustRegex {
    /// Create a new regex from a pattern.
    ///
    /// This expects normal Rust regex syntax.
    #[new]
    pub fn new(patter: &str) -> PyResult<Self> {
        let inner = Regex::new(patter)
            .map_err(|e| PyValueError::new_err(format!("Invalid regex: {}", e)))?;
        let names = Arc::new(
            inner
                .capture_names()
                .map(|n| n.map(|n| n.to_owned()))
                .collect(),
        );
        Ok(Self { inner, names })
    }

    #[getter]
    pub fn pattern(&self) -> &str {
        self.inner.as_str()
    }
    #[getter]
    pub fn groups(&self) -> usize {
        self.names.len() - 1
    }
    #[getter]
    pub fn groupindex(&self) -> HashMap<String, usize> {
        self.names
            .iter()
            .enumerate()
            .filter_map(|(i, n)| n.as_ref().map(|n| (n.clone(), i)))
            .collect()
    }

    pub fn search(&self, text: &str, pos: Option<usize>) -> Option<RustRegexMatch> {
        let mut translator = PosTranslator::new(text);
        self.inner
            .captures_at(text, to_byte_pos(text, pos.unwrap_or(0)))
            .map(|m| RustRegexMatch::from_captures(&mut translator, m, self.names.clone()))
    }

    pub fn findall(&self, text: &str) -> Vec<RustRegexMatch> {
        let mut translator = PosTranslator::new(text);
        self.inner
            .captures_iter(text)
            .map(|m| RustRegexMatch::from_captures(&mut translator, m, self.names.clone()))
            .collect()
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

#[pyclass(frozen)]
#[derive(Debug)]
pub struct RustRegexMatch {
    groups: Vec<Option<RustRegexGroup>>,
    names: Arc<Vec<Option<String>>>,
}

impl RustRegexMatch {
    fn first(&self) -> &RustRegexGroup {
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
                    g.map(|g| RustRegexGroup {
                        start: translator.get_char_pos(g.start()),
                        end: translator.get_char_pos(g.end()),
                    })
                })
                .collect(),
            names,
        }
    }
}

#[pymethods]
impl RustRegexMatch {
    #[getter]
    pub fn start(&self) -> usize {
        self.first().start
    }
    #[getter]
    pub fn end(&self) -> usize {
        self.first().end
    }
    #[getter]
    pub fn len(&self) -> usize {
        let first = self.first();
        first.end - first.start
    }

    pub fn get(&self, index: usize) -> Option<RustRegexGroup> {
        self.groups.get(index).cloned().flatten()
    }

    pub fn get_by_name(&self, name: &str) -> Option<RustRegexGroup> {
        self.names
            .iter()
            .position(|n| n.as_ref().map(|n| n.as_str()) == Some(name))
            .and_then(|i| self.groups.get(i).cloned().flatten())
    }
}

#[pyclass(frozen, get_all)]
#[derive(Debug, Clone, PartialEq)]
pub struct RustRegexGroup {
    pub start: usize,
    pub end: usize,
}

#[pymethods]
impl RustRegexGroup {
    #[getter]
    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_props() {
        assert_eq!(RustRegex::new(r"foo").unwrap().pattern(), "foo");
        assert_eq!(RustRegex::new(r"foo").unwrap().groups(), 0);
        assert_eq!(RustRegex::new(r"(foo)").unwrap().pattern(), "(foo)");
        assert_eq!(RustRegex::new(r"(foo)").unwrap().groups(), 1);
    }

    #[test]
    fn regex_search() {
        let re = RustRegex::new(r"\b(fo+)").unwrap();

        let m: RustRegexMatch = re.search("foo", None).unwrap();
        assert_eq!(m.start(), 0);
        assert_eq!(m.end(), 3);
        assert_eq!(m.len(), 3);
        assert_eq!(m.get(0), Some(RustRegexGroup { start: 0, end: 3 }));
    }

    #[test]
    fn regex_search_unicode() {
        let re = RustRegex::new(r"\B.").unwrap();

        let m: RustRegexMatch = re.search("äöü", None).unwrap();
        assert_eq!(m.start(), 1);
        assert_eq!(m.end(), 2);
        assert_eq!(m.len(), 1);
        assert_eq!(m.get(0), Some(RustRegexGroup { start: 1, end: 2 }));
    }
}
