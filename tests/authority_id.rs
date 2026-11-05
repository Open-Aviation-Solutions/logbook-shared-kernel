// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use icao_shared_kernel::{Licence, ValidationError};
use logbook_shared_kernel::AuthorityId;

#[test]
fn an_australian_licence_is_issued_by_au_casa() {
    let licence = Licence::new("AU", "CASA", "123456", Vec::new()).unwrap();
    let id = AuthorityId::of(&licence);
    assert_eq!(id, AuthorityId::new("AU", "CASA").unwrap());
    assert_eq!((id.state(), id.authority()), ("AU", "CASA"));
    assert_eq!(id.to_string(), "AU-CASA");
}

#[test]
fn what_a_licence_refuses_of_its_state_and_authority_is_refused() {
    for state in ["", "A", "AUS", "au", "A1"] {
        assert_eq!(
            AuthorityId::new(state, "CASA"),
            Err(ValidationError::IssuingState(state.into())),
            "{state}"
        );
        assert!(Licence::new(state, "CASA", "1", Vec::new()).is_err());
    }
    assert_eq!(
        AuthorityId::new("AU", ""),
        Err(ValidationError::Empty {
            field: "issuing_authority"
        })
    );
    assert!(Licence::new("AU", "", "1", Vec::new()).is_err());
}

#[test]
fn the_uks_caa_and_new_zealands_are_two_authorities() {
    let uk = AuthorityId::new("GB", "CAA").unwrap();
    let nz = AuthorityId::new("NZ", "CAA").unwrap();
    assert_ne!(uk, nz);
    assert_eq!(uk.to_string(), "GB-CAA");
}
