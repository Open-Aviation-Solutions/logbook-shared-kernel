// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The fields of the common entry that an authority may leave unasked, and
//! the evaluation of when they are asked
//! ([`LicensingAuthority::entry_fields`]), so that the host and the
//! authority read the conditions the same way.

use std::collections::BTreeSet;

use super::{LicensingAuthority, TermId};
use crate::entry::{Entry, Record};

/// A field of the common entry that an authority may leave unasked, on a
/// condition ([`LicensingAuthority::entry_fields`]).
///
/// Non-exhaustive: a field is added when an authority's rule leaves it out,
/// and that is not a breaking change, as a host shows a field it does not
/// know of here always.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum EntryField {
    /// [`Entry::instructor`].
    Instructor,
    /// [`Entry::examiner`].
    Examiner,
}

/// Whether `field` is asked of `entry`, under the authorities in `under`:
/// those whose rules the host asks about this entry (the authorities of the
/// pilot's licences, say). Each is given with what the pilot held under the
/// licences it issued on the entry's date, as
/// [`LicensingAuthority::requirements_for`] takes it (the host's to give, as
/// it is for that): one set of term ids, of titles of licence, ratings and
/// endorsements alike, so a [`Condition::Holds`](super::Condition::Holds)
/// is met by any of them, a title's included.
///
/// It is asked when any of the authorities asks it: one that gives no
/// condition for the field, or one with a condition for it that holds for
/// `entry` and what is held under it. With no authority, none asks it, so
/// it is not asked. This is the only way to evaluate the conditions, so
/// that the host and the authority read them the same way.
///
/// It decides only what is asked, not what is valid: a value in a field
/// that is not asked is still the entry's, and the host shows the field
/// when the entry holds one.
pub fn entry_field_applies<'a>(
    field: EntryField,
    entry: &Entry,
    under: impl IntoIterator<Item = (&'a dyn LicensingAuthority, &'a BTreeSet<TermId>)>,
) -> bool {
    under.into_iter().any(|(authority, held)| {
        let mut conditions = authority
            .entry_fields()
            .into_iter()
            .filter(|(of, _)| *of == field)
            .map(|(_, condition)| condition)
            .peekable();
        conditions.peek().is_none()
            || conditions
                .any(|condition| condition.holds(Record::Entry(entry), &|_| None, Some(held)))
    })
}
