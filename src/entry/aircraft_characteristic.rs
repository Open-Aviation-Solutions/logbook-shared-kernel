// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::BTreeSet;

use aircraft_type_descriptions::{AircraftDescription, AirframeKind};

/// A physical fact about an aircraft's airframe that the description of its
/// type does not give, and that some authority's rules turn on. A fact the
/// description already gives (its engines, say) stays in the description.
///
/// ICAO's Annex 1 sets no list of these: each authority draws its own
/// constructs from them (CASA a design feature endorsement, the FAA an
/// endorsement or a class, EASA a variant or a class), and says what an
/// aircraft with them requires through
/// [`LicensingAuthority::classify`]. Each variant cites the rules that use
/// it; one is added only when an authority needs it.
///
/// [`LicensingAuthority::classify`]: crate::authority::LicensingAuthority::classify
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AircraftCharacteristic {
    /// CASA's "tailwheel undercarriage" (CASR 1998 reg 61.755(2)(a)); the
    /// FAA's "tailwheel airplanes" (14 CFR 61.31(i)); EASA's "with tail
    /// wheels" (GM1 FCL.700).
    TailwheelUndercarriage,
    /// CASA's "retractable undercarriage" (reg 61.755(2)(b), (3)(b),
    /// (3A)(a), (4)(a)); the FAA's "retractable landing gear", part of a
    /// "complex airplane" (14 CFR 61.1(b)); EASA's "with retractable
    /// undercarriage" (GM1 FCL.700).
    RetractableUndercarriage,
    /// CASA's "manual propeller pitch control (piston engine)" (reg
    /// 61.755(2)(c)). It overlaps the FAA's "controllable pitch propeller"
    /// (14 CFR 61.1(b)) and EASA's "with variable pitch propellers" (GM1
    /// FCL.700) without being the same set of aircraft.
    ManualPropellerPitchControl,
    /// CASA's "multi-engine centre-line thrust" (reg 61.755(2)(e)), whose
    /// aeroplanes are in the single-engine aeroplane class (reg
    /// 61.020(2)(a)).
    CentreLineThrust,
    /// CASA's "pressurisation system" (reg 61.755(2)(f), (3A)(b), (4)(b),
    /// (5)(a)); the FAA's "pressurized aircraft" (14 CFR 61.31(g)), which
    /// it draws only above a 25,000 ft ceiling; EASA's "with cabin
    /// pressurisation" (GM1 FCL.700).
    PressurisationSystem,
    /// Floats, on which the aircraft operates on water: CASA's
    /// "floatplane" for an aeroplane (reg 61.755(2)(g)) and "float
    /// alighting gear" for a helicopter (reg 61.755(3)(a)). For the FAA
    /// (14 CFR 61.5(b)(2)) and EASA (GM1 FCL.700), operating on water is a
    /// class.
    Floats,
    /// CASA's "floating hull" (reg 61.755(2)(h)).
    FloatingHull,
    /// CASA's "ski landing gear" (reg 61.755(2)(i)).
    Skis,
}

impl AircraftCharacteristic {
    /// The characteristics an aircraft of a type has unless its own record
    /// says otherwise: what the type's description implies. A host that
    /// records an aircraft's characteristics uses the recorded set in place
    /// of this one, not added to it.
    ///
    /// A seaplane or an amphibian implies floats; for one with a floating
    /// hull, its record says so.
    pub fn implied_by(description: &AircraftDescription) -> BTreeSet<Self> {
        match description.airframe {
            AirframeKind::SeaPlane | AirframeKind::Amphibian => BTreeSet::from([Self::Floats]),
            AirframeKind::LandPlane
            | AirframeKind::Helicopter
            | AirframeKind::Gyrocopter
            | AirframeKind::TiltRotor
            | AirframeKind::Glider
            | AirframeKind::Airship => BTreeSet::new(),
        }
    }
}
