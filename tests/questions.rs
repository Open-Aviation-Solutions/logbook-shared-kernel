// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An authority's questions, tested with pilot-logbook's ADR 0001's two
//! cases. Each set of questions is a fixture here only: this crate holds no
//! State's rules.

mod common;

use common::{assessment, flight, flight_parts, session};
use icao_shared_kernel::{ApproachType, Capacity, LocationIndicator};
use logbook_shared_kernel::{
    Answer, Approach, AuthorityId, AuthoritySpecificInformation, Choice, Condition, Entry,
    Question, Questions, Record,
};
use serde_json::{json, Value};

/// CASA's pilot in command under supervision (CASR 1998 reg 61.095), as the
/// ADR describes its information. The ADR gives the choices' labels; the
/// questions' are words of reg 61.095(2)(b) and (3)(b). No label is
/// asserted.
fn casa_picus() -> Questions {
    let picus = || {
        Some(Condition::Capacity(
            Capacity::PilotInCommandUnderSupervision,
        ))
    };
    Questions::new(
        1,
        vec![
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
    )
}

/// EASA's pilot-in-command under supervision (FCL.010; AMC1 FCL.050),
/// written only to test the description, and not built.
fn easa_picus() -> Questions {
    Questions::new(
        1,
        vec![Question {
            key: "countersigned_by",
            label: "Countersigned by the PIC".into(),
            regulation: "AMC1 FCL.050(b)(1)(v), (b)(5)",
            answer: Answer::Person,
            shown_when: Some(Condition::Capacity(
                Capacity::PilotInCommandUnderSupervision,
            )),
            required: true,
        }],
    )
}

/// The keys of the questions shown for an entry flown in `capacity`.
fn shown(
    questions: &Questions,
    capacity: Capacity,
    information: Option<Value>,
) -> Vec<&'static str> {
    let entry = flight("e", capacity);
    let information = information.map(AuthoritySpecificInformation::new);
    questions
        .shown_for(Record::Entry(&entry), information.as_ref())
        .into_iter()
        .map(|question| question.key)
        .collect()
}

const PICUS: Capacity = Capacity::PilotInCommandUnderSupervision;

#[test]
fn casas_questions_are_not_shown_for_a_pilot_in_command() {
    assert!(shown(&casa_picus(), Capacity::PilotInCommand, None).is_empty());
}

#[test]
fn casas_questions_ask_the_route_first() {
    assert_eq!(shown(&casa_picus(), PICUS, None), ["route"]);
}

#[test]
fn casas_questions_ask_about_flight_training_only_on_the_instructor_route() {
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
fn the_adrs_information_answers_every_question_casa_shows() {
    let information = json!({"route": "instructor", "receiving_flight_training": false});
    let questions = casa_picus();
    assert_answers_every_question_shown(&questions, PICUS, &information, 2);
}

/// Each question shown, `count` of them, has an answer of its kind in
/// `information`.
fn assert_answers_every_question_shown(
    questions: &Questions,
    capacity: Capacity,
    information: &Value,
    count: usize,
) {
    let entry = flight("e", capacity);
    let information = AuthoritySpecificInformation::new(information.clone());
    let shown = questions.shown_for(Record::Entry(&entry), Some(&information));
    assert_eq!(shown.len(), count);
    for question in shown {
        let answer = &information.value()[question.key];
        assert!(question.answer.accepts(answer), "{}", question.key);
    }
}

#[test]
fn easas_question_is_shown_only_for_pilot_in_command_under_supervision() {
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
    let information = json!({"countersigned_by": {"name": "A. Pilot"}});
    assert_answers_every_question_shown(&easa_picus(), PICUS, &information, 1);
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
    let questions = Questions::new(
        1,
        vec![
            question("first", Condition::Capacity(Capacity::PilotInCommand)),
            question(
                "second",
                Condition::Answer {
                    key: "first",
                    is: json!(true),
                },
            ),
        ],
    );
    let information = Some(json!({"first": true}));
    assert_eq!(
        shown(&questions, Capacity::PilotInCommand, information.clone()),
        ["first", "second"]
    );
    assert!(shown(&questions, PICUS, information).is_empty());
}

#[test]
fn a_condition_on_a_yes_or_no_answer_matches_the_boolean_stored() {
    let questions = Questions::new(
        1,
        vec![
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
    );
    let pic = Capacity::PilotInCommand;
    assert_eq!(
        shown(&questions, pic, Some(json!({"first": false}))),
        ["first", "second"]
    );
    for information in [json!({"first": true}), json!({"first": "false"}), json!({})] {
        assert_eq!(shown(&questions, pic, Some(information)), ["first"]);
    }
    assert_eq!(shown(&questions, pic, None), ["first"]);
}

#[test]
fn a_hidden_follow_ups_answer_is_removed_and_the_rest_kept() {
    let questions = casa_picus();
    let entry = flight("e", PICUS);
    let mut information = AuthoritySpecificInformation::new(json!({
        "route": "operator",
        "receiving_flight_training": false,
        "other": 1,
        "_version": 1
    }));
    questions.drop_hidden_answers_for(Record::Entry(&entry), &mut information);
    assert_eq!(
        information.value(),
        &json!({"route": "operator", "other": 1, "_version": 1})
    );
}

#[test]
fn the_required_unanswered_questions_are_those_shown_without_an_accepted_answer() {
    let questions = casa_picus();
    let entry = flight("e", PICUS);
    let unanswered = |information: Option<Value>| -> Vec<&'static str> {
        let information = information.map(AuthoritySpecificInformation::new);
        questions
            .required_unanswered_for(Record::Entry(&entry), information.as_ref())
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
fn the_questions_version_is_stored_with_the_answers_and_read_back() {
    let questions = Questions::new(2, Vec::new());
    let mut information =
        AuthoritySpecificInformation::new(json!({"route": "operator", "_version": 1}));
    questions.stamp(&mut information);
    assert_eq!(
        information.value(),
        &json!({"route": "operator", "_version": 2})
    );
    assert_eq!(information.version(), Some(questions.version()));
}

#[test]
fn stamping_null_information_gives_it_only_the_version() {
    let mut information = AuthoritySpecificInformation::new(Value::Null);
    casa_picus().stamp(&mut information);
    assert_eq!(information.value(), &json!({"_version": 1}));
}

#[test]
fn information_without_a_whole_number_under_the_key_has_no_version() {
    for value in [
        json!({}),
        json!({"_version": "1"}),
        json!({"_version": 1.5}),
        json!({"_version": -1}),
        json!({"_version": u64::from(u32::MAX) + 1}),
        json!([1]),
    ] {
        assert_eq!(
            AuthoritySpecificInformation::new(value.clone()).version(),
            None,
            "{value}"
        );
    }
}

/// Questions with one always shown, a follow-up on its answer, and one for
/// a pilot in command.
fn with_capacity_question() -> Questions {
    let question = |key, shown_when| Question {
        key,
        label: key.into(),
        regulation: "",
        answer: Answer::YesNo,
        shown_when,
        required: true,
    };
    Questions::new(
        1,
        vec![
            question("first", None),
            question(
                "second",
                Some(Condition::Answer {
                    key: "first",
                    is: json!(true),
                }),
            ),
            question("pic", Some(Condition::Capacity(Capacity::PilotInCommand))),
        ],
    )
}

#[test]
fn a_capacity_condition_holds_only_for_an_entry() {
    let questions = with_capacity_question();
    let entry = flight("e", Capacity::PilotInCommand);
    let assessment = assessment("a");
    let keys = |record| -> Vec<&'static str> {
        questions
            .shown_for(record, None)
            .into_iter()
            .map(|question| question.key)
            .collect()
    };
    assert_eq!(keys(Record::Entry(&entry)), ["first", "pic"]);
    assert_eq!(keys(Record::Assessment(&assessment)), ["first"]);
}

#[test]
fn an_assessments_answers_show_remove_and_leave_unanswered_as_an_entrys_do() {
    let questions = with_capacity_question();
    let assessment = assessment("a");
    let record = Record::Assessment(&assessment);
    let keys = |questions: Vec<&Question>| -> Vec<&'static str> {
        questions.into_iter().map(|question| question.key).collect()
    };
    let mut information =
        AuthoritySpecificInformation::new(json!({"first": false, "second": true, "pic": true}));
    assert_eq!(
        keys(questions.shown_for(record, Some(&information))),
        ["first"]
    );
    questions.drop_hidden_answers_for(record, &mut information);
    assert_eq!(information.value(), &json!({"first": false}));
    let information = AuthoritySpecificInformation::new(json!({"first": true}));
    assert_eq!(
        keys(questions.required_unanswered_for(record, Some(&information))),
        ["second"]
    );
}

#[test]
fn a_records_information_is_found_by_its_authority() {
    let authority = AuthorityId::new("XX", "TEST").unwrap();
    let other = AuthorityId::new("YY", "TEST").unwrap();
    let information = AuthoritySpecificInformation::new(json!({"first": true}));
    let mut parts = flight_parts("e", Capacity::PilotInCommand);
    parts
        .authority_information
        .insert(authority.clone(), information.clone());
    let entry = Entry::new(parts).unwrap();
    let mut assessment = assessment("a");
    assessment
        .authority_information
        .insert(authority.clone(), information.clone());
    for record in [Record::Entry(&entry), Record::Assessment(&assessment)] {
        assert_eq!(record.information_for(&authority), Some(&information));
        assert_eq!(record.information_for(&other), None);
    }
}

/// Questions with one for a session and one for an entry with approaches.
fn with_record_questions() -> Questions {
    let question = |key, shown_when| Question {
        key,
        label: key.into(),
        regulation: "",
        answer: Answer::YesNo,
        shown_when: Some(shown_when),
        required: false,
    };
    Questions::new(
        1,
        vec![
            question("session", Condition::Session),
            question("approaches", Condition::Approaches),
        ],
    )
}

#[test]
fn a_record_condition_holds_only_for_an_entry_that_is_a_session_or_records_an_approach() {
    let questions = with_record_questions();
    let flight = flight("e", Capacity::PilotInCommand);
    let session = session("s", Capacity::PilotInCommand);
    let mut parts = flight_parts("a", Capacity::PilotInCommand);
    parts.approaches.push(Approach::new(
        ApproachType::Ils,
        LocationIndicator::parse("YSBK").unwrap(),
        false,
    ));
    let approached = Entry::new(parts).unwrap();
    let assessment = assessment("x");
    let keys = |record| -> Vec<&'static str> {
        questions
            .shown_for(record, None)
            .into_iter()
            .map(|question| question.key)
            .collect()
    };
    assert!(keys(Record::Entry(&flight)).is_empty());
    assert_eq!(keys(Record::Entry(&session)), ["session"]);
    assert_eq!(keys(Record::Entry(&approached)), ["approaches"]);
    assert!(keys(Record::Assessment(&assessment)).is_empty());
}

#[test]
fn an_answer_left_after_the_record_changed_is_removed() {
    let questions = with_record_questions();
    let entry = flight("e", Capacity::PilotInCommand);
    let mut information =
        AuthoritySpecificInformation::new(json!({"session": true, "approaches": true}));
    questions.drop_hidden_answers_for(Record::Entry(&entry), &mut information);
    assert_eq!(information.value(), &json!({}));
}
