// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt;

/// An authority's refusal to migrate its information on a record
/// ([`LicensingAuthority::migrate_information`]). The kind says what the host
/// advises; the message, in the authority's words (the key at fault, say),
/// is what it shows with that advice.
///
/// Not exhaustive: a refusal that asks the pilot to decide something can be
/// added without breaking a host.
///
/// [`LicensingAuthority::migrate_information`]: crate::authority::LicensingAuthority::migrate_information
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MigrationRefused {
    /// The information is of a newer version than the rules write: it was
    /// saved by a newer app. The host says to update this app.
    Newer(String),
    /// The information is older than every migration the authority keeps.
    /// The host says to update in steps.
    TooOld(String),
    /// The information does not read as the version it claims, or has none:
    /// the record is damaged. The host says its file needs correcting.
    Unreadable(String),
}

impl MigrationRefused {
    /// Why the information was refused, for the host to show.
    pub fn message(&self) -> &str {
        match self {
            Self::Newer(message) | Self::TooOld(message) | Self::Unreadable(message) => message,
        }
    }
}

impl fmt::Display for MigrationRefused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for MigrationRefused {}
