// Copyright (c) 2018, [Ribose Inc](https://www.ribose.com).
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// ``AS IS'' AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::collections::{BTreeMap, HashMap};
use std::iter::{self, FromIterator};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Dict(HashMap<String, Value>),
    List(Vec<Value>),
}

impl<'a> From<&'a str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_owned())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(m: HashMap<String, Value>) -> Self {
        Value::Dict(m)
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::List(v)
    }
}

impl Value {
    pub fn insert<'a, I, V>(&mut self, keys: I, value: V)
    where
        I: IntoIterator<Item = &'a str>,
        V: Into<Value>,
    {
        let value = value.into();
        let mut keys = keys.into_iter().peekable();
        let key = keys.next().unwrap();
        let map = self.as_dict_mut().unwrap();
        let old_value = map.remove(key).filter(|v| v.is_dict());

        map.insert(
            key.to_owned(),
            if keys.peek().is_none() {
                // single key so insert in current node
                match (value, old_value) {
                    (Value::Dict(mut new), Some(Value::Dict(mut existing))) => {
                        for (k, v) in new.drain() {
                            existing.insert(k, v);
                        }
                        Value::Dict(existing)
                    }
                    (v, _) => v,
                }
            } else {
                let mut node = old_value.unwrap_or_else(|| Value::Dict(HashMap::new()));
                node.insert(keys.collect::<Vec<_>>(), value);
                node
            },
        );
    }

    pub fn get<T>(&self, key: &str) -> Result<T, String>
    where
        T: FromValue,
    {
        self.as_dict().map_or_else(
            || Err("Value is not a dict".to_owned()),
            |dict| {
                dict.get(key)
                    .map_or_else(|| T::from_no_value(), T::from_value)
            },
        )
    }

    pub fn get_value<'a, I>(&'a self, key: &str) -> Result<&'a Value, String> {
        self.as_dict()
            .ok_or_else(|| "Not a dict".to_owned())
            .and_then(|d| d.get(key).ok_or_else(|| "No such key".to_owned()))
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_ref()),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Value::String(_) => true,
            _ => false,
        }
    }

    pub fn into_string(self) -> Option<String> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_dict<'a>(&'a self) -> Option<&'a HashMap<String, Value>> {
        match self {
            Value::Dict(ref map) => Some(map),
            _ => None,
        }
    }

    pub fn as_dict_mut<'a>(&'a mut self) -> Option<&'a mut HashMap<String, Value>> {
        match self {
            Value::Dict(ref mut map) => Some(map),
            _ => None,
        }
    }

    pub fn is_dict(&self) -> bool {
        match self {
            Value::Dict(_) => true,
            _ => false,
        }
    }

    pub fn as_list<'a>(&'a self) -> Option<&'a Vec<Value>> {
        match self {
            Value::List(ref vec) => Some(vec),
            _ => None,
        }
    }

    pub fn as_list_mut<'a>(&'a mut self) -> Option<&'a mut Vec<Value>> {
        match self {
            Value::List(ref mut vec) => Some(vec),
            _ => None,
        }
    }

    pub fn is_list(&self) -> bool {
        match self {
            Value::List(_) => true,
            _ => false,
        }
    }

    pub fn as_noc_string(&self) -> String {
        match self {
            Value::String(s) => format!("\"{}\"", s),
            Value::List(v) => {
                let values = v
                    .iter()
                    .map(|v| match v {
                        Value::Dict(_) => format!("{{{}}}", v.as_noc_string()),
                        Value::List(_) => format!("[{}]", v.as_noc_string()),
                        Value::String(_) => v.as_noc_string(),
                    })
                    .collect::<Vec<_>>();
                values.join(",")
            }
            Value::Dict(m) => {
                let values = BTreeMap::from_iter(m.iter())
                    .iter()
                    .map(|(k, v)| match v {
                        Value::Dict(_) => format!("\"{}\" {{{}}}", k, v.as_noc_string()),
                        Value::List(_) => format!("\"{}\" [{}]", k, v.as_noc_string()),
                        Value::String(_) => format!("\"{}\" {}", k, v.as_noc_string()),
                    })
                    .collect::<Vec<_>>();
                values.join(",")
            }
        }
    }

    pub fn as_noc_string_pretty(&self) -> String {
        self.as_s_indent(0)
    }

    fn as_s_indent(&self, indent: usize) -> String {
        let tabs = iter::repeat('\t').take(indent).collect::<String>();
        match self {
            Value::String(s) => format!("\"{}\"", s),
            Value::List(v) => v
                .iter()
                .map(|v| {
                    let s = v.as_s_indent(indent + 1);
                    match v {
                        Value::Dict(_) => format!("{}{{\n{}\n{}}}", tabs, s, tabs),
                        Value::List(_) => format!("{}[\n{}\n{}]", tabs, s, tabs),
                        Value::String(_) => format!("{}{}", tabs, s),
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Value::Dict(m) => BTreeMap::from_iter(m.iter()) // sort
                .iter()
                .map(|(k, v)| {
                    let s = v.as_s_indent(indent + 1);
                    match v {
                        Value::Dict(_) => format!("{}\"{}\" {{\n{}\n{}}}", tabs, k, s, tabs),
                        Value::List(_) => format!("{}\"{}\" [\n{}\n{}]", tabs, k, s, tabs),
                        Value::String(_) => format!("{}\"{}\" {}", tabs, k, s),
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

impl FromStr for Value {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, String> {
        super::parse(input)
    }
}

pub trait FromValue<OK = Self> {
    fn from_value(value: &Value) -> Result<OK, String>;
    // this is a kludge so Value::get::<Option<T>> can work
    fn from_no_value() -> Result<OK, String> {
        Err("No such key".to_owned())
    }
}

impl FromValue for String {
    fn from_value(value: &Value) -> Result<Self, String> {
        value
            .as_str()
            .map(String::from)
            .map_or_else(|| Err("Value is not a string".to_owned()), Ok)
    }
}

impl<T> FromValue for Option<T>
where
    T: FromValue,
{
    fn from_value(value: &Value) -> Result<Self, String> {
        T::from_value(value).map(Some).or_else(|_| Ok(None))
    }
    fn from_no_value() -> Result<Self, String> {
        Ok(None)
    }
}

impl<T, S: ::std::hash::BuildHasher + Default> FromValue for HashMap<String, T, S>
where
    T: FromValue,
{
    fn from_value(value: &Value) -> Result<Self, String> {
        value
            .as_dict()
            .ok_or_else(|| "Couldn't convert".to_owned())
            .and_then(|d| {
                d.iter().try_fold(HashMap::default(), |mut m, (k, v)| {
                    T::from_value(v).map(|v| {
                        m.insert(k.to_owned(), v);
                        m
                    })
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::Value;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    fn test_value_from() {
        assert_eq!(Value::from(HashMap::new()), Value::Dict(HashMap::new()));
        assert_eq!(Value::from(Vec::new()), Value::List(Vec::new()));
        assert_eq!(Value::from("hello"), Value::String("hello".to_owned()));
        assert_eq!(
            Value::from("hello".to_owned()),
            Value::String("hello".to_owned())
        );
    }

    #[test]
    fn test_value_insert() {
        let mut v = Value::from(HashMap::new());
        v.insert(vec!["a"], "a");
        assert_eq!(v.as_noc_string(), r#""a" "a""#);
        v.insert(vec!["b"], "b");
        assert_eq!(v.as_noc_string(), r#""a" "a","b" "b""#);
        v.insert(vec!["c", "c", "c"], "c");
        assert_eq!(v.as_noc_string(), r#""a" "a","b" "b","c" {"c" {"c" "c"}}"#);
        v.insert(vec!["c"], "c");
        assert_eq!(v.as_noc_string(), r#""a" "a","b" "b","c" "c""#);
    }

    #[test]
    fn test_value_get() {
        let value = Value::from_str(r#"a a, b b, c c, e {}, f []"#).unwrap();
        assert_eq!(value.get("a"), Ok("a".to_owned()));
        assert_eq!(value.get("a"), Ok(Some("a".to_owned())));
        assert_eq!(value.get("b"), Ok("b".to_owned()));
        assert_eq!(value.get("c"), Ok("c".to_owned()));
        assert!(value.get::<String>("d").is_err());
        assert_eq!(value.get::<Option<String>>("d"), Ok(None));
        assert_eq!(
            value.get::<HashMap<String, String>>("e"),
            Ok(HashMap::new())
        );
    }
}
