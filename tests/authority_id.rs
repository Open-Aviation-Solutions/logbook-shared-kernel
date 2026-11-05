// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use icao_shared_kernel::{IssuingState, Licence, ValidationError};
use logbook_shared_kernel::AuthorityId;

#[test]
fn an_australian_licence_is_issued_by_au_casa() {
    let licence = Licence::new(
        IssuingState::parse("AU").unwrap(),
        "CASA",
        "123456",
        Vec::new(),
    )
    .unwrap();
    let id = AuthorityId::of(&licence);
    assert_eq!(id, AuthorityId::new("AU", "CASA").unwrap());
    assert_eq!((id.state(), id.authority()), ("AU", "CASA"));
    assert_eq!(id.to_string(), "AU-CASA");
}

#[test]
fn a_bad_state_or_an_empty_authority_is_refused() {
    for state in ["", "A", "AUS", "au", "A1"] {
        assert_eq!(
            AuthorityId::new(state, "CASA"),
            Err(ValidationError::IssuingState(state.into())),
            "{state}"
        );
    }
    assert_eq!(
        AuthorityId::new("AU", ""),
        Err(ValidationError::Empty {
            field: "issuing_authority"
        })
    );
}

#[test]
fn the_uks_caa_and_new_zealands_are_two_authorities() {
    let uk = AuthorityId::new("GB", "CAA").unwrap();
    let nz = AuthorityId::new("NZ", "CAA").unwrap();
    assert_ne!(uk, nz);
    assert_eq!(uk.to_string(), "GB-CAA");
}
