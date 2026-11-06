// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The form description, tested with pilot-logbook's ADR 0001's two cases.
//! Each form is a fixture here only: this crate holds no State's rules.

mod common;

use common::flight;
use icao_shared_kernel::Capacity;
use logbook_shared_kernel::{Answer, Choice, Condition, Form, Question, Section};
use serde_json::{json, Value};

/// CASA's pilot in command under supervision (CASR 1998 reg 61.095), as the
/// ADR describes its section. The ADR gives the choices' labels; the
/// questions' are words of reg 61.095(2)(b) and (3)(b). No label is
/// asserted.
fn casa_picus() -> Form {
    let picus = || {
        Some(Condition::Capacity(
            Capacity::PilotInCommandUnderSupervision,
        ))
    };
    Form {
        version: 1,
        questions: vec![
            Question {
                key: "route",
                label: "The supervision of the person".into(),
                regulation: "61.095(1)(c)",
                answer: Answer::Choice(vec![
                    Choice {
                        value: "operator",
                        label: "The flight is conducted by an operator that has training and \
                                checking responsibilities, and the pilot in command is \
                                authorised by the operator or its Part 142 operator to \
                                conduct the supervision"
                            .into(),
                        regulation: "61.095(2)",
                    },
                    Choice {
                        value: "instructor",
                        label: "You are supervised by a flight instructor or flight examiner"
                            .into(),
                        regulation: "61.095(3)",
                    },
                ]),
                shown_when: picus(),
                required: true,
            },
            Question {
                key: "receiving_flight_training",
                label: "Receiving flight training".into(),
                regulation: "61.095(3)(b)",
                answer: Answer::YesNo,
                shown_when: Some(Condition::All(vec![
                    picus().unwrap(),
                    Condition::Answer {
                        key: "route",
                        is: json!("instructor"),
                    },
                ])),
                required: true,
            },
        ],
    }
}

/// EASA's pilot-in-command under supervision (FCL.010; AMC1 FCL.050),
/// written only to test the description, and not built.
fn easa_picus() -> Form {
    Form {
        version: 1,
        questions: vec![Question {
            key: "countersigned_by",
            label: "Countersigned by the PIC".into(),
            regulation: "AMC1 FCL.050(b)(1)(v), (b)(5)",
            answer: Answer::Person,
            shown_when: Some(Condition::Capacity(
                Capacity::PilotInCommandUnderSupervision,
            )),
            required: true,
        }],
    }
}

/// The keys of the questions shown for an entry flown in `capacity`.
fn shown(form: &Form, capacity: Capacity, section: Option<Value>) -> Vec<&'static str> {
    let entry = flight("e", capacity);
    let section = section.map(Section::new);
    form.shown(&entry, section.as_ref())
        .into_iter()
        .map(|question| question.key)
        .collect()
}

const PICUS: Capacity = Capacity::PilotInCommandUnderSupervision;

#[test]
fn casas_form_is_not_shown_for_a_pilot_in_command() {
    assert!(shown(&casa_picus(), Capacity::PilotInCommand, None).is_empty());
}

#[test]
fn casas_form_asks_the_route_first() {
    assert_eq!(shown(&casa_picus(), PICUS, None), ["route"]);
}

#[test]
fn casas_form_asks_about_flight_training_only_on_the_instructor_route() {
    assert_eq!(
        shown(&casa_picus(), PICUS, Some(json!({"route": "instructor"}))),
        ["route", "receiving_flight_training"]
    );
    assert_eq!(
        shown(&casa_picus(), PICUS, Some(json!({"route": "operator"}))),
        ["route"]
    );
}

#[test]
fn the_adrs_section_answers_every_question_casa_shows() {
    let section = json!({"route": "instructor", "receiving_flight_training": false});
    let form = casa_picus();
    assert_answers_every_question_shown(&form, PICUS, &section, 2);
}

/// Each question shown, `count` of them, has an answer of its kind in
/// `section`.
fn assert_answers_every_question_shown(
    form: &Form,
    capacity: Capacity,
    section: &Value,
    count: usize,
) {
    let entry = flight("e", capacity);
    let section = Section::new(section.clone());
    let questions = form.shown(&entry, Some(&section));
    assert_eq!(questions.len(), count);
    for question in questions {
        let answer = &section.value()[question.key];
        assert!(question.answer.accepts(answer), "{}", question.key);
    }
}

#[test]
fn easas_form_is_shown_only_for_pilot_in_command_under_supervision() {
    for capacity in [
        Capacity::PilotInCommand,
        Capacity::CoPilot,
        Capacity::DualInstruction,
    ] {
        assert!(
            shown(&easa_picus(), capacity, None).is_empty(),
            "{capacity:?}"
        );
    }
    assert_eq!(shown(&easa_picus(), PICUS, None), ["countersigned_by"]);
}

#[test]
fn a_person_named_answers_easas_question() {
    let section = json!({"countersigned_by": {"name": "A. Pilot"}});
    assert_answers_every_question_shown(&easa_picus(), PICUS, &section, 1);
}

#[test]
fn an_answer_is_accepted_only_in_its_kinds_stored_shape() {
    let choice = Answer::Choice(vec![Choice {
        value: "one",
        label: "One".into(),
        regulation: "",
    }]);
    assert!(choice.accepts(&json!("one")));
    assert!(!choice.accepts(&json!("two")));
    assert!(Answer::YesNo.accepts(&json!(false)));
    assert!(!Answer::YesNo.accepts(&json!("false")));
    assert!(Answer::Text.accepts(&json!("Text")));
    assert!(!Answer::Text.accepts(&json!("")));
    assert!(!Answer::Text.accepts(&json!("  ")));
    assert!(!Answer::Text.accepts(&json!(1)));
    assert!(Answer::Person.accepts(&json!({"name": "A. Pilot", "licence_number": "1"})));
    for person in [
        json!("A. Pilot"),
        json!({"licence_number": "1"}),
        json!({"name": "A. Pilot", "licence_number": 1}),
        json!({"name": "A. Pilot", "signature": "A. P."}),
    ] {
        assert!(!Answer::Person.accepts(&person), "{person}");
    }
    for count in [json!(0), json!(1), json!(12)] {
        assert!(Answer::Count.accepts(&count), "{count}");
    }
    for count in [
        json!(-1),
        json!(1.5),
        json!(2.0),
        json!("2"),
        json!(true),
        json!(null),
        json!([1]),
        json!({"count": 1}),
    ] {
        assert!(!Answer::Count.accepts(&count), "{count}");
    }
}

#[test]
fn a_persons_licence_number_may_be_null() {
    assert!(Answer::Person.accepts(&json!({"name": "A. Pilot", "licence_number": null})));
}

#[test]
fn a_persons_name_may_not_be_empty() {
    for name in ["", "  "] {
        assert!(
            !Answer::Person.accepts(&json!({ "name": name })),
            "{name:?}"
        );
    }
}

#[test]
fn an_answer_to_a_hidden_question_meets_no_condition() {
    let question = |key, shown_when| Question {
        key,
        label: key.into(),
        regulation: "",
        answer: Answer::YesNo,
        shown_when: Some(shown_when),
        required: true,
    };
    let form = Form {
        version: 1,
        questions: vec![
            question("first", Condition::Capacity(Capacity::PilotInCommand)),
            question(
                "second",
                Condition::Answer {
                    key: "first",
                    is: json!(true),
                },
            ),
        ],
    };
    let section = Some(json!({"first": true}));
    assert_eq!(
        shown(&form, Capacity::PilotInCommand, section.clone()),
        ["first", "second"]
    );
    assert!(shown(&form, PICUS, section).is_empty());
}

#[test]
fn a_condition_on_a_yes_or_no_answer_matches_the_boolean_stored() {
    let form = Form {
        version: 1,
        questions: vec![
            Question {
                key: "first",
                label: "First".into(),
                regulation: "",
                answer: Answer::YesNo,
                shown_when: None,
                required: true,
            },
            Question {
                key: "second",
                label: "Second".into(),
                regulation: "",
                answer: Answer::Text,
                shown_when: Some(Condition::Answer {
                    key: "first",
                    is: json!(false),
                }),
                required: false,
            },
        ],
    };
    let pic = Capacity::PilotInCommand;
    assert_eq!(
        shown(&form, pic, Some(json!({"first": false}))),
        ["first", "second"]
    );
    for section in [json!({"first": true}), json!({"first": "false"}), json!({})] {
        assert_eq!(shown(&form, pic, Some(section)), ["first"]);
    }
    assert_eq!(shown(&form, pic, None), ["first"]);
}

#[test]
fn a_hidden_follow_ups_answer_is_removed_and_the_rest_kept() {
    let form = casa_picus();
    let entry = flight("e", PICUS);
    let mut section = Section::new(json!({
        "route": "operator",
        "receiving_flight_training": false,
        "other": 1,
        "version": 1
    }));
    form.retain_shown(&entry, &mut section);
    assert_eq!(
        section.value(),
        &json!({"route": "operator", "other": 1, "version": 1})
    );
}

#[test]
fn the_unanswered_questions_are_those_shown_and_required_without_an_accepted_answer() {
    let form = casa_picus();
    let entry = flight("e", PICUS);
    let unanswered = |section: Option<Value>| -> Vec<&'static str> {
        let section = section.map(Section::new);
        form.unanswered(&entry, section.as_ref())
            .into_iter()
            .map(|question| question.key)
            .collect()
    };
    assert_eq!(unanswered(None), ["route"]);
    assert_eq!(unanswered(Some(json!({"route": "elsewhere"}))), ["route"]);
    assert_eq!(
        unanswered(Some(json!({"route": "instructor"}))),
        ["receiving_flight_training"]
    );
    assert_eq!(
        unanswered(Some(
            json!({"route": "instructor", "receiving_flight_training": "no"})
        )),
        ["receiving_flight_training"]
    );
    assert!(unanswered(Some(json!({"route": "operator"}))).is_empty());
    assert!(unanswered(Some(
        json!({"route": "instructor", "receiving_flight_training": false})
    ))
    .is_empty());
}

#[test]
fn the_forms_version_is_stored_with_the_answers_and_read_back() {
    let mut form = casa_picus();
    form.version = 2;
    let mut section = Section::new(json!({"route": "operator", "version": 1}));
    form.stamp(&mut section);
    assert_eq!(section.value(), &json!({"route": "operator", "version": 2}));
    assert_eq!(section.version(), Some(2));
}

#[test]
fn stamping_a_null_section_gives_it_only_the_version() {
    let mut section = Section::new(Value::Null);
    casa_picus().stamp(&mut section);
    assert_eq!(section.value(), &json!({"version": 1}));
}

#[test]
fn a_section_without_a_whole_number_under_the_key_has_no_version() {
    for value in [
        json!({}),
        json!({"version": "1"}),
        json!({"version": 1.5}),
        json!({"version": -1}),
        json!({"version": u64::from(u32::MAX) + 1}),
        json!([1]),
    ] {
        assert_eq!(Section::new(value.clone()).version(), None, "{value}");
    }
}
