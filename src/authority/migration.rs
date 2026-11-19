// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt;

/// An authority's refusal to migrate its information on a record
/// ([`LicensingAuthority::migrate_information`]): the information is not of
/// any version its rules read, including one older than every migration it
/// keeps, or it cannot be read as the version it claims.
///
/// Its message says why, in the authority's words, for the host to show; the
/// host does not act on it by kind. Not exhaustive: a refusal that asks the
/// pilot to decide something can be added without breaking a host.
///
/// [`LicensingAuthority::migrate_information`]: crate::authority::LicensingAuthority::migrate_information
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MigrationRefused {
    message: String,
}

impl MigrationRefused {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Why the information was refused, for the host to show.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for MigrationRefused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for MigrationRefused {}
