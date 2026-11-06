// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! What an authority's section is about.

use super::{Assessment, Entry};

/// A record in the logbook that holds an authority's section: what the
/// section, and the authority's form for it, are about. The host says which
/// at each call, and the authority answers for each kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Record<'a> {
    Entry(&'a Entry),
    Assessment(&'a Assessment),
}
