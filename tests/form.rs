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
/// ADR describes its section. The labels of the questions themselves are not
/// the ADR's, and are not asserted.
fn casa_picus() -> Form {
    let picus = || {
        Some(Condition::Capacity(
            Capacity::PilotInCommandUnderSupervision,
        ))
    };
    Form {
        questions: vec![
            Question {
                key: "route",
                label: "Method of supervision".into(),
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
    let keys = shown(&form, PICUS, Some(section.clone()));
    assert_eq!(keys.len(), 2);
    for key in keys {
        assert!(section.get(key).is_some(), "{key}");
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
fn a_condition_on_a_yes_or_no_answer_matches_the_boolean_stored() {
    let form = Form {
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
