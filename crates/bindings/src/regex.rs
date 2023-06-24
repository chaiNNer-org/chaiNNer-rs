use pyo3::{exceptions::PyValueError, prelude::*};
use std::{collections::HashMap, ops::Range};

#[pyclass(frozen)]
pub struct RustRegex {
    inner: regex_py::Regex,
}

#[pymethods]
impl RustRegex {
    /// Create a new regex from a pattern.
    ///
    /// This expects normal Rust regex syntax.
    #[new]
    pub fn new(patter: &str) -> PyResult<Self> {
        let inner = regex_py::Regex::new(patter).map_err(|e| PyValueError::new_err(e.message))?;
        Ok(Self { inner })
    }

    #[getter]
    pub fn pattern(&self) -> &str {
        self.inner.pattern()
    }
    #[getter]
    pub fn groups(&self) -> usize {
        self.inner.groups()
    }
    #[getter]
    pub fn groupindex(&self) -> HashMap<String, usize> {
        self.inner.groupindex()
    }

    pub fn search(&self, text: &str, pos: Option<usize>) -> Option<RegexMatch> {
        self.inner.search(text, pos.unwrap_or(0)).map(|m| m.into())
    }

    pub fn findall(&self, text: &str) -> Vec<RegexMatch> {
        self.inner.findall(text).map(|m| m.into()).collect()
    }

    pub fn split(&self, text: &str) -> Vec<String> {
        self.inner.split(text)
    }

    pub fn split_without_captures(&self, text: &str) -> Vec<String> {
        self.inner.split_without_captures(text)
    }
}

#[pyclass(frozen)]
#[derive(Debug)]
pub struct RegexMatch {
    inner: regex_py::RegexMatch,
}

#[pymethods]
impl RegexMatch {
    #[getter]
    pub fn start(&self) -> usize {
        self.inner.start()
    }
    #[getter]
    pub fn end(&self) -> usize {
        self.inner.end()
    }
    #[getter]
    pub fn len(&self) -> usize {
        self.inner.end() - self.inner.start()
    }

    pub fn get(&self, index: usize) -> Option<MatchGroup> {
        self.inner.get(index).map(|r| r.into())
    }

    pub fn get_by_name(&self, name: &str) -> Option<MatchGroup> {
        self.inner.get_by_name(name).map(|r| r.into())
    }
}

impl From<regex_py::RegexMatch> for RegexMatch {
    fn from(r: regex_py::RegexMatch) -> Self {
        Self { inner: r }
    }
}
#[pyclass(frozen, get_all)]
#[derive(Debug, Clone, PartialEq)]
pub struct MatchGroup {
    pub start: usize,
    pub end: usize,
}

#[pymethods]
impl MatchGroup {
    #[getter]
    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

impl From<Range<usize>> for MatchGroup {
    fn from(r: Range<usize>) -> Self {
        Self {
            start: r.start,
            end: r.end,
        }
    }
}
