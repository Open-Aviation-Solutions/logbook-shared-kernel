// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// A physical fact about an aircraft's airframe that the description of its
/// type does not give, and that some authority's rules turn on. A fact the
/// description already gives (its engines, say) stays in the description.
///
/// ICAO's Annex 1 sets no list of these: each authority draws its own
/// constructs from them (CASA a design feature endorsement, the FAA an
/// endorsement, EASA a variant), and says what an aircraft with them
/// requires through [`LicensingAuthority::classify`]. Each is named in the
/// words of a regulation that uses it, and cites the rules that do; one is
/// added only when an authority needs it.
///
/// Nothing implies them: an aircraft has the characteristics recorded for
/// it, or, for a session, the characteristics recorded for the configuration
/// its device represented (`Simulated`), a copy of its own and not linked to an
/// aircraft. Its type's description cannot tell them apart (a seaplane's float
/// alighting gear from a floating hull, say).
///
/// [`LicensingAuthority::classify`]: crate::authority::LicensingAuthority::classify
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AircraftCharacteristic {
    /// CASA's "tailwheel undercarriage" (CASR 1998 reg 61.755(2)(a)); the
    /// FAA's "tailwheel airplane" (14 CFR 61.31(i)); EASA's "SEP (land)
    /// with tail wheels" (GM1 FCL.700, table (a)).
    TailwheelUndercarriage,
    /// CASA's "retractable undercarriage" (reg 61.755(2)(b), (3)(b),
    /// (3A)(a), (4)(a)); the FAA's "retractable landing gear", part of a
    /// "complex airplane" (14 CFR 61.1(b)); EASA's "SEP (land) with
    /// retractable undercarriage" (GM1 FCL.700, table (a)).
    RetractableUndercarriage,
    /// CASA's "manual propeller pitch control (piston engine)" (reg
    /// 61.755(2)(c)). It overlaps the FAA's "controllable pitch propeller",
    /// part of a "complex airplane" (14 CFR 61.1(b)), and EASA's "with
    /// variable pitch propellers" (GM1 FCL.700, table (a)) without being
    /// the same set of aircraft.
    ManualPropellerPitchControl,
    /// CASA's "multi-engine centre-line thrust" (reg 61.755(2)(e)), whose
    /// aeroplanes are in the single-engine aeroplane class (reg
    /// 61.020(2)(a)).
    MultiEngineCentreLineThrust,
    /// CASA's "pressurisation system" (reg 61.755(2)(f), (3A)(b), (4)(b),
    /// (5)(a)); the FAA's "pressurized aircraft" (14 CFR 61.31(g)), which
    /// it draws only above a service ceiling or maximum operating altitude
    /// of 25,000 feet MSL; EASA's "with cabin pressurisation" (GM1 FCL.700,
    /// table (a)).
    PressurisationSystem,
    /// CASA's "float alighting gear" (reg 61.755(3)(a)), on which the
    /// aircraft operates on water, for any category: it also covers the
    /// aeroplane CASA calls a "floatplane" (reg 61.755(2)(g)).
    FloatAlightingGear,
    /// CASA's "floating hull" (reg 61.755(2)(h)).
    FloatingHull,
    /// CASA's "ski landing gear" (reg 61.755(2)(i)).
    SkiLandingGear,
}
