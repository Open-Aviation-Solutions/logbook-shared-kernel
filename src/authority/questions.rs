// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The questions an authority asks about a record (an entry or an
//! assessment), described as data for the host to show, with no dependence
//! on a user interface.
//!
//! A question is shown on conditions on the record and on the answers
//! already given, which this module evaluates, so that the host and the
//! authority read them the same way. For the same reason it says which
//! answers are stale ([`Questions::drop_hidden_answers_for`]) and which
//! required questions are unanswered
//! ([`Questions::required_unanswered_for`]). The rules that refuse an answer
//! are the authority's
//! [`problems_for`](super::LicensingAuthority::problems_for), in code.

use icao_shared_kernel::Capacity;
use serde_json::Value;

use crate::entry::{AuthoritySpecificInformation, Record};

/// The questions an authority asks about a record, in order, and the
/// version of the stored format of their answers.
#[derive(Debug, Clone, PartialEq)]
pub struct Questions {
    version: u32,
    questions: Vec<Question>,
}

impl Questions {
    /// `version` is the version of the stored format of the answers, the
    /// authority's own. It changes it with each breaking change to that
    /// format, which ships with a migration of the stored information. The
    /// host stores it with the answers ([`Questions::stamp`]), and the
    /// authority reads it back from stored information
    /// ([`AuthoritySpecificInformation::version`]). Each set of questions
    /// carries its own, so an authority's information on entries and on
    /// assessments is versioned independently.
    ///
    /// No question's key may be [`AuthoritySpecificInformation::VERSION_KEY`].
    /// Nothing checks the questions for this.
    pub fn new(version: u32, questions: Vec<Question>) -> Self {
        Self { version, questions }
    }

    /// Every question, in order, whether shown or not.
    pub fn iter(&self) -> std::slice::Iter<'_, Question> {
        self.questions.iter()
    }

    /// Write the version into `information`, under
    /// [`AuthoritySpecificInformation::VERSION_KEY`], replacing any there. A
    /// host calls it last, after the authority has checked the information
    /// as stored, and just before it saves it, so that the authority sees
    /// stored information's own version (and new information's none) and the
    /// saved information says which version of the questions its answers
    /// were given to. Information that is `null` becomes an object holding
    /// only the version; any other that is not a JSON object is left as it
    /// is, and so has no version.
    pub fn stamp(&self, information: &mut AuthoritySpecificInformation) {
        let value = information.value_mut();
        if value.is_null() {
            *value = Value::Object(Default::default());
        }
        if let Some(answers) = value.as_object_mut() {
            answers.insert(
                AuthoritySpecificInformation::VERSION_KEY.into(),
                self.version.into(),
            );
        }
    }

    /// The questions shown for `record`, in order, given the answers in
    /// `information` so far (`None`: none yet).
    ///
    /// An answer counts towards a condition only when its question is shown
    /// before the one the condition is on: an answer left in the information
    /// to a question now hidden is not given, so its follow-ups are hidden
    /// too. This is the only way to evaluate the questions' conditions, so
    /// that the host and the authority read them the same way.
    pub fn shown_for<'a>(
        &'a self,
        record: Record,
        information: Option<&AuthoritySpecificInformation>,
    ) -> Vec<&'a Question> {
        let mut shown: Vec<&'a Question> = Vec::new();
        for question in &self.questions {
            let answer = |key: &str| {
                shown
                    .iter()
                    .any(|earlier| earlier.key == key)
                    .then(|| information.and_then(|information| information.value().get(key)))
                    .flatten()
            };
            if question
                .shown_when
                .as_ref()
                .is_none_or(|condition| condition.holds(record, &answer))
            {
                shown.push(question);
            }
        }
        shown
    }

    /// Remove from `information` the answers to these questions that
    /// [`Questions::shown_for`] does not show for `record`: an answer left
    /// from before the pilot changed the answer that showed its question. A
    /// host calls it before it asks the authority for the problems with the
    /// information or saves it, so that the authority never reads an answer
    /// the pilot cannot see. A key that is no question's is kept, as it is
    /// the authority's, and so is the version.
    pub fn drop_hidden_answers_for(
        &self,
        record: Record,
        information: &mut AuthoritySpecificInformation,
    ) {
        let shown: Vec<&str> = self
            .shown_for(record, Some(information))
            .into_iter()
            .map(|question| question.key)
            .collect();
        if let Some(answers) = information.value_mut().as_object_mut() {
            for question in &self.questions {
                if !shown.contains(&question.key) {
                    answers.remove(question.key);
                }
            }
        }
    }

    /// The questions shown for `record` that are required and have no
    /// answer: their key is absent from `information`, or holds a value that
    /// the question's [`Answer::accepts`] refuses. This is what "answered"
    /// means, for the host and the authority alike.
    pub fn required_unanswered_for<'a>(
        &'a self,
        record: Record,
        information: Option<&AuthoritySpecificInformation>,
    ) -> Vec<&'a Question> {
        self.shown_for(record, information)
            .into_iter()
            .filter(|question| question.required)
            .filter(|question| {
                information
                    .and_then(|information| information.value().get(question.key))
                    .is_none_or(|answer| !question.answer.accepts(answer))
            })
            .collect()
    }
}

/// One question.
#[derive(Debug, Clone, PartialEq)]
pub struct Question {
    /// The key its answer is stored under in the authority-specific
    /// information.
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
/// authority-specific information, under the question's key, so that the host and the authority
/// agree on it.
#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    /// One of the choices, stored as its `value`, a string.
    Choice(Vec<Choice>),
    /// Stored as a boolean.
    YesNo,
    /// Stored as a string, which is not empty or only whitespace: an empty
    /// text is no answer, so the host stores none.
    Text,
    /// A person, stored as `{"name": ..., "licence_number": ...}`: the name
    /// a string, and required, so not empty or only whitespace; the licence
    /// number a string, and optional (absent, or `null`).
    Person,
    /// A non-negative whole number, stored as a JSON number written as an
    /// integer literal (`0`, `3`), with no fraction part or exponent: `2.0`
    /// and `2e0` are not an answer, nor is a negative or fractional number
    /// or any other type. A writer in another language must emit an integer.
    /// No count is no answer, so the host stores no key (not `0`).
    Count,
}

impl Answer {
    /// Whether `value`, as stored in authority-specific information, is an
    /// answer of this kind.
    pub fn accepts(&self, value: &Value) -> bool {
        match self {
            Self::Choice(choices) => value
                .as_str()
                .is_some_and(|value| choices.iter().any(|choice| choice.value == value)),
            Self::YesNo => value.is_boolean(),
            Self::Text => value.as_str().is_some_and(|text| !text.trim().is_empty()),
            Self::Person => value.as_object().is_some_and(|person| {
                person
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| !name.trim().is_empty())
                    && person
                        .get("licence_number")
                        .is_none_or(|number| number.is_string() || number.is_null())
                    && person
                        .keys()
                        .all(|key| key == "name" || key == "licence_number")
            }),
            Self::Count => value.is_u64(),
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
///
/// A condition on an answer may refer only to an earlier question:
/// [`Questions::shown_for`] counts only the answers to questions already
/// shown, so a condition on a later question's key, or on a key no question
/// has, is never met, and the question it is on is never shown. Nothing
/// checks the questions for this.
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    /// The record is an entry, and its capacity is this one. It never
    /// holds for an assessment, which has no capacity.
    Capacity(Capacity),
    /// The answer stored under `key` is `is`, compared as it is stored.
    /// Only the answer to a question shown earlier counts.
    Answer { key: &'static str, is: Value },
    /// Every one of these.
    All(Vec<Condition>),
}

impl Condition {
    /// Whether the condition holds, with `answer` giving the answer to each
    /// key, if given.
    fn holds<'s>(&self, record: Record, answer: &dyn Fn(&str) -> Option<&'s Value>) -> bool {
        match self {
            Self::Capacity(capacity) => {
                matches!(record, Record::Entry(entry) if entry.capacity() == *capacity)
            }
            Self::Answer { key, is } => answer(key) == Some(is),
            Self::All(conditions) => conditions
                .iter()
                .all(|condition| condition.holds(record, answer)),
        }
    }
}
