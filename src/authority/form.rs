// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The form description: the questions an authority asks about an entry, as
//! data for the host to show, with no dependence on a user interface.
//!
//! A question is shown on conditions on the entry's common part and on the
//! answers already given, which this module evaluates, so that the host and
//! the authority read them the same way. The rules that refuse an answer are
//! the authority's [`validate_section`](super::LicensingAuthority::validate_section),
//! in code.

use icao_shared_kernel::Capacity;
use serde_json::Value;

use crate::entry::{Entry, Section};

/// The questions an authority asks about an entry, in order.
#[derive(Debug, Clone, PartialEq)]
pub struct Form {
    pub questions: Vec<Question>,
}

impl Form {
    /// The questions shown for an entry, in order, given the answers in
    /// `section` so far (`None`: none yet).
    ///
    /// An answer counts towards a condition only when its question is shown
    /// before the one the condition is on: an answer left in the section to
    /// a question now hidden is not given, so its follow-ups are hidden too.
    pub fn shown<'a>(&'a self, entry: &Entry, section: Option<&Section>) -> Vec<&'a Question> {
        let mut shown: Vec<&'a Question> = Vec::new();
        for question in &self.questions {
            let answer = |key: &str| {
                shown
                    .iter()
                    .any(|earlier| earlier.key == key)
                    .then(|| section.and_then(|section| section.value().get(key)))
                    .flatten()
            };
            if question
                .shown_when
                .as_ref()
                .is_none_or(|condition| condition.holds(entry, &answer))
            {
                shown.push(question);
            }
        }
        shown
    }
}

/// One question.
#[derive(Debug, Clone, PartialEq)]
pub struct Question {
    /// The key its answer is stored under in the section.
    pub key: &'static str,
    /// In the regulation's words.
    pub label: String,
    /// What it is asked for.
    pub regulation: &'static str,
    pub answer: Answer,
    /// `None`: always shown.
    pub shown_when: Option<Condition>,
    pub required: bool,
}

/// The kind of answer a question takes, and how it is stored in the
/// section, under the question's key, so that the host and the authority
/// agree on it.
#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    /// One of the choices, stored as its `value`, a string.
    Choice(Vec<Choice>),
    /// Stored as a boolean.
    YesNo,
    /// Stored as a string.
    Text,
    /// A person, stored as `{"name": ..., "licence_number": ...}`: the name
    /// a string, and required; the licence number a string, and optional.
    Person,
}

impl Answer {
    /// Whether `value`, as stored in a section, is an answer of this kind.
    pub fn accepts(&self, value: &Value) -> bool {
        match self {
            Self::Choice(choices) => value
                .as_str()
                .is_some_and(|value| choices.iter().any(|choice| choice.value == value)),
            Self::YesNo => value.is_boolean(),
            Self::Text => value.is_string(),
            Self::Person => value.as_object().is_some_and(|person| {
                person.get("name").is_some_and(Value::is_string)
                    && person.get("licence_number").is_none_or(Value::is_string)
                    && person
                        .keys()
                        .all(|key| key == "name" || key == "licence_number")
            }),
        }
    }
}

/// One choice of a [`Answer::Choice`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// What is stored when it is chosen.
    pub value: &'static str,
    /// In the regulation's words.
    pub label: String,
    pub regulation: &'static str,
}

/// When a question is shown.
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    /// The entry's capacity is this one.
    Capacity(Capacity),
    /// The answer stored under `key` is `is`, compared as it is stored.
    /// Through [`Form::shown`], only the answer to a question shown earlier
    /// counts; [`Condition::is_met`] reads the section as it is.
    Answer { key: &'static str, is: Value },
    /// Every one of these.
    All(Vec<Condition>),
}

impl Condition {
    /// Whether the condition holds for an entry, given the answers in
    /// `section` so far. An answer not given meets no condition on it.
    pub fn is_met(&self, entry: &Entry, section: Option<&Section>) -> bool {
        self.holds(entry, &|key| {
            section.and_then(|section| section.value().get(key))
        })
    }

    /// Whether the condition holds, with `answer` giving the answer to each
    /// key, if given.
    fn holds<'s>(&self, entry: &Entry, answer: &dyn Fn(&str) -> Option<&'s Value>) -> bool {
        match self {
            Self::Capacity(capacity) => entry.capacity() == *capacity,
            Self::Answer { key, is } => answer(key) == Some(is),
            Self::All(conditions) => conditions
                .iter()
                .all(|condition| condition.holds(entry, answer)),
        }
    }
}
