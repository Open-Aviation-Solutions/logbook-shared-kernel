// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The accumulator abstractions and the single-pass runners.

use super::LicensingAuthority;
use crate::entry::{Assessment, Entry};

/// Anything that folds logbook entries into a result.
///
/// **Order independence is part of the contract.** A result depends on the
/// set of entries accepted, never on the order they arrive in, so any
/// accumulator can be driven by any iteration order (a database cursor, a
/// directory listing) without changing its answer. Implementations that keep
/// dates sort or count them rather than relying on arrival order.
pub trait EntryAccumulator {
    fn accept_entry(&mut self, entry: &Entry);
}

/// Dispatch each entry that counts under an authority's rules to every
/// accumulator, in a single pass.
///
/// This is the one place the authority's [`counts`] is applied: an entry it
/// refuses is dropped here, so no accumulator guards against it, and it
/// reaches no check or total either.
///
/// [`counts`]: LicensingAuthority::counts
pub fn run<'a, A: LicensingAuthority + ?Sized>(
    authority: &A,
    entries: impl IntoIterator<Item = &'a Entry>,
    accumulators: &mut [&mut dyn EntryAccumulator],
) {
    for entry in entries {
        if !authority.counts(entry) {
            continue;
        }
        for accumulator in accumulators.iter_mut() {
            accumulator.accept_entry(entry);
        }
    }
}

/// Anything that folds assessments into a result.
///
/// Separate from [`EntryAccumulator`]: a check driven by assessments has
/// nothing to do with entries. The same order independence contract applies:
/// a result depends on the set of assessments accepted, not the order they
/// arrive in.
pub trait AssessmentAccumulator {
    fn accept_assessment(&mut self, assessment: &Assessment);
}

/// Dispatch each assessment to every accumulator in a single pass.
pub fn run_assessments<'a>(
    assessments: impl IntoIterator<Item = &'a Assessment>,
    accumulators: &mut [&mut dyn AssessmentAccumulator],
) {
    for assessment in assessments {
        for accumulator in accumulators.iter_mut() {
            accumulator.accept_assessment(assessment);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use time::macros::date;
    use uuid::Uuid;

    use super::*;
    use crate::entry::AssessmentId;

    /// The ids of the assessments it was given.
    #[derive(Default)]
    struct Seen(Vec<AssessmentId>);

    impl AssessmentAccumulator for Seen {
        fn accept_assessment(&mut self, assessment: &Assessment) {
            self.0.push(assessment.id.clone());
        }
    }

    #[test]
    fn one_assessment_reaches_every_assessment_accumulator_given_it() {
        let assessment = Assessment {
            id: AssessmentId::new("a"),
            date: date!(2024 - 08 - 02),
            flight: Uuid::nil(),
            authority_information: BTreeMap::new(),
        };
        let (mut first, mut second) = (Seen::default(), Seen::default());
        run_assessments(
            &[assessment],
            &mut [
                &mut first as &mut dyn AssessmentAccumulator,
                &mut second as &mut dyn AssessmentAccumulator,
            ],
        );
        assert_eq!(first.0, [AssessmentId::new("a")]);
        assert_eq!(second.0, [AssessmentId::new("a")]);
    }
}
