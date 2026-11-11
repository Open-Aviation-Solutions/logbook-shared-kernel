// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The questions an authority asks about a record (an entry or an
//! assessment), described as data for the host to show, with no dependence
//! on a user interface.
//!
//! A question applies to a record on conditions on the record and on the
//! answers already given, which this module evaluates, so that the host and
//! the authority read them the same way. For the same reason it says which
//! answers are stale ([`Questions::retain_applicable_answers_for`]) and which
//! required questions are unanswered
//! ([`Questions::required_unanswered_for`]). The rules that refuse an answer
//! are the authority's
//! [`problems_for`](super::LicensingAuthority::problems_for), in code.

use icao_shared_kernel::Capacity;
use serde_json::Value;

use crate::entry::{AuthoritySpecificInformation, Record, Subject};

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

    /// The version of the stored format of the answers, for the authority
    /// to compare with [`AuthoritySpecificInformation::version`].
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Every question, in order, whether it applies or not.
    pub fn iter(&self) -> std::slice::Iter<'_, Question> {
        self.questions.iter()
    }

    /// Whether there are no questions: the authority asks nothing about the
    /// record.
    pub fn is_empty(&self) -> bool {
        self.questions.is_empty()
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

    /// The questions that apply to `record`, in order, given the answers in
    /// `information` so far (`None`: none yet).
    ///
    /// An answer counts towards a condition only when its question applies
    /// and comes before the one the condition is on: an answer left in the
    /// information to a question that no longer applies is not given, so its
    /// follow-ups do not apply either. This is the only way to evaluate the
    /// questions' conditions, so that the host and the authority read them
    /// the same way.
    pub fn applicable_for<'a>(
        &'a self,
        record: Record,
        information: Option<&AuthoritySpecificInformation>,
    ) -> Vec<&'a Question> {
        let mut applicable: Vec<&'a Question> = Vec::new();
        for question in &self.questions {
            let answer = |key: &str| {
                applicable
                    .iter()
                    .any(|earlier| earlier.key == key)
                    .then(|| information.and_then(|information| information.value().get(key)))
                    .flatten()
            };
            if question
                .applies_when
                .as_ref()
                .is_none_or(|condition| condition.holds(record, &answer))
            {
                applicable.push(question);
            }
        }
        applicable
    }

    /// Keep in `information` only the answers to these questions that apply
    /// to `record` ([`Questions::applicable_for`]), removing the rest: the
    /// answers to questions that no longer apply because the pilot changed
    /// the record, or an earlier answer, since answering them. A host calls it before it asks the
    /// authority for the problems with the information or saves it, so that
    /// the authority never reads an answer to a question that does not
    /// apply. A key that is no question's is kept, as it is the authority's,
    /// and so is the version.
    ///
    /// So a question asked only of some records belongs in the list, with a
    /// [`Condition`] on the record, not left out of it: only then is its
    /// answer removed when the record changes.
    pub fn retain_applicable_answers_for(
        &self,
        record: Record,
        information: &mut AuthoritySpecificInformation,
    ) {
        let applicable: Vec<&str> = self
            .applicable_for(record, Some(information))
            .into_iter()
            .map(|question| question.key)
            .collect();
        if let Some(answers) = information.value_mut().as_object_mut() {
            for question in &self.questions {
                if !applicable.contains(&question.key) {
                    answers.remove(question.key);
                }
            }
        }
    }

    /// The questions that apply to `record` that are required and have no
    /// answer: their key is absent from `information`, or holds a value that
    /// the question's [`Answer::accepts`] refuses. This is what "answered"
    /// means, for the host and the authority alike.
    pub fn required_unanswered_for<'a>(
        &'a self,
        record: Record,
        information: Option<&AuthoritySpecificInformation>,
    ) -> Vec<&'a Question> {
        self.applicable_for(record, information)
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

impl<'a> IntoIterator for &'a Questions {
    type Item = &'a Question;
    type IntoIter = std::slice::Iter<'a, Question>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
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
    /// `None`: it always applies.
    pub applies_when: Option<Condition>,
    pub required: bool,
}

/// The kind of answer a question takes, and how it is stored in the
/// authority-specific information, under the question's key, so that the
/// host and the authority agree on it.
///
/// Unlike [`Condition`], it is not `#[non_exhaustive]`: a host matches on it
/// to draw each question, so a kind of answer added later is a breaking
/// release, and a host does not build until it draws the new kind.
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
    /// Several of the choices in `groups`, each answered further with one
    /// of `each`, stored as a list of
    /// `{"kind": ..., "value": ..., "answer": ...}`: the chosen one's group's
    /// kind's `value`, its own `value`, and the `value` of its answer among
    /// `each`, all strings, and no other key. The items are stored in the
    /// order of the choices, group by group, so that an answer has only one
    /// spelling. It is an answer when it holds at least one item, each
    /// item's kind and value are those of a choice in `groups`, the items are
    /// in that order with none listed twice, and each is answered with one
    /// of `each`. No choice is no answer, so the host stores no key (not
    /// `[]`).
    ///
    /// A [`Condition::Answer`] on such a question compares the whole list.
    ///
    /// `each` and each group's choices must not be empty (with no `each`, a
    /// required question can never be answered), and a kind's `value` must
    /// be one group's only, and a choice's `value` must appear only once in
    /// its group. Nothing checks the questions for this.
    ///
    /// A host draws its parts in `labels`' words, and labels the choice
    /// within a group with the group's kind's label.
    Choices {
        groups: Vec<ChoiceGroup>,
        each: Vec<Choice>,
        labels: ChoicesLabels,
    },
}

impl Answer {
    /// Whether `value`, as stored in authority-specific information, is an
    /// answer of this kind.
    pub fn accepts(&self, value: &Value) -> bool {
        match self {
            Self::Choice(choices) => value
                .as_str()
                .is_some_and(|value| Choice::is_one_of(choices, value)),
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
            Self::Choices { groups, each, .. } => value
                .as_array()
                .is_some_and(|items| Self::accepts_choices(groups, each, items)),
        }
    }

    /// Whether `items` answer an [`Answer::Choices`] of the choices in
    /// `groups`, each answered with one of `each`, in the choices' order.
    fn accepts_choices(groups: &[ChoiceGroup], each: &[Choice], items: &[Value]) -> bool {
        let offered: Vec<(&str, &str)> = groups
            .iter()
            .flat_map(|group| {
                group
                    .choices
                    .iter()
                    .map(|choice| (group.kind.value, choice.value))
            })
            .collect();
        let mut previous: Option<usize> = None;
        for item in items {
            let Some((kind, value, answer)) = Self::kinded_choice(item) else {
                return false;
            };
            let Some(position) = offered.iter().position(|&chosen| chosen == (kind, value)) else {
                return false;
            };
            if previous.is_some_and(|previous| position <= previous)
                || !Choice::is_one_of(each, answer)
            {
                return false;
            }
            previous = Some(position);
        }
        previous.is_some()
    }

    /// An item of an [`Answer::Choices`] as stored, read as its kind, value
    /// and answer, if it has those keys, each a string, and no other.
    fn kinded_choice(item: &Value) -> Option<(&str, &str, &str)> {
        let item = item.as_object()?;
        if item.len() != 3 {
            return None;
        }
        let field = |key| item.get(key).and_then(Value::as_str);
        Some((field("kind")?, field("value")?, field("answer")?))
    }
}

/// One choice of an [`Answer::Choice`], one of a [`ChoiceGroup`]'s, or
/// one of the answers to each choice of an [`Answer::Choices`] (its `each`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// What is stored when it is chosen.
    pub value: &'static str,
    /// In the regulation's words.
    pub label: String,
    pub regulation: &'static str,
    /// Whether choosing it is adverse to the pilot, such as a failed
    /// outcome, so that a host can set it apart without spelling an
    /// authority's value. It means the same wherever a choice is used: in an
    /// [`Answer::Choice`], in a [`ChoiceGroup`], or in the `each` of an
    /// [`Answer::Choices`].
    pub adverse: bool,
}

impl Choice {
    /// Whether `value` is the stored value of one of `choices`.
    fn is_one_of(choices: &[Choice], value: &str) -> bool {
        choices.iter().any(|choice| choice.value == value)
    }
}

/// The labels of the parts of an [`Answer::Choices`] that a host draws, in
/// the regulation's words. The choice within a group has none of its own: it
/// is labelled by its group's [`ChoiceKind`]'s label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoicesLabels {
    /// What one item is, in the singular, so that a host can draw
    /// "Add a `item`".
    pub item: String,
    /// The label of each item's answer, its choice among `each`.
    pub answer: String,
}

/// The choices of one kind in an [`Answer::Choices`], by which a host may
/// group them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceGroup {
    pub kind: ChoiceKind,
    pub choices: Vec<Choice>,
}

/// The kind of the choices in a [`ChoiceGroup`], which is stored with each
/// one chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceKind {
    /// What is stored as the kind of a choice of it.
    pub value: &'static str,
    /// In the regulation's words.
    pub label: String,
}

/// When a question applies to a record.
///
/// A condition on an answer may refer only to an earlier question:
/// [`Questions::applicable_for`] counts only the answers to earlier
/// questions that apply, so a condition on a later question's key, or on a
/// key no question has, is never met, and the question it is on never
/// applies. Nothing checks the questions for this.
///
/// It is `#[non_exhaustive]`, so that a condition added later is not a
/// breaking release: only this crate evaluates conditions. A consumer that
/// uses a condition added in a patch requires that patch (`^0.4.1`, say), as
/// a lock file holding an earlier one does not build it.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Condition {
    /// The record is an entry, and its capacity is this one. It never
    /// holds for an assessment, which has no capacity.
    Capacity(Capacity),
    /// The record is an entry of a session in a flight simulation training
    /// device ([`Subject::Session`]). It never holds for an assessment.
    Session,
    /// The record is an entry that records at least one instrument approach
    /// ([`Entry::approaches`](crate::entry::Entry::approaches)). It never
    /// holds for an assessment.
    AnyApproach,
    /// The answer stored under `key` is `is`, compared as it is stored.
    /// Only the answer to an earlier question that applies counts.
    Answer { key: &'static str, is: Value },
    /// The answer stored under `key` is a count ([`Answer::Count`]) of at
    /// least `n`. A missing answer, or one that is not a count, does not
    /// meet it. Only the answer to an earlier question that applies counts.
    CountAtLeast { key: &'static str, n: u64 },
    /// Every one of these. With none, it always holds.
    All(Vec<Condition>),
    /// Any one of these. With none, it never holds.
    Any(Vec<Condition>),
}

impl Condition {
    /// Whether the condition holds, with `answer` giving the answer to each
    /// key, if given.
    fn holds<'s>(&self, record: Record, answer: &dyn Fn(&str) -> Option<&'s Value>) -> bool {
        match self {
            Self::Capacity(capacity) => {
                matches!(record, Record::Entry(entry) if entry.capacity() == *capacity)
            }
            Self::Session => match record {
                Record::Entry(entry) => matches!(entry.subject(), Subject::Session { .. }),
                Record::Assessment(_) => false,
            },
            Self::AnyApproach => {
                matches!(record, Record::Entry(entry) if !entry.approaches().is_empty())
            }
            Self::Answer { key, is } => answer(key) == Some(is),
            Self::CountAtLeast { key, n } => answer(key)
                .filter(|value| Answer::Count.accepts(value))
                .and_then(Value::as_u64)
                .is_some_and(|count| count >= *n),
            Self::All(conditions) => conditions
                .iter()
                .all(|condition| condition.holds(record, answer)),
            Self::Any(conditions) => conditions
                .iter()
                .any(|condition| condition.holds(record, answer)),
        }
    }
}
