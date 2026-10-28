# logbook-shared-kernel

A Rust library of the part of a pilot's logbook that the logbook and each
licensing authority's rules agree on. A logbook records each flight in the
terms ICAO's Annex 1 defines (flight time, instrument flight time,
instrument ground time, night, pilot-in-command, co-pilot, pilot-in-command
under supervision, dual instruction time and solo flight time), and each
authority adds to them, narrows them and checks them under its own
regulations. This crate holds the common part, and the interface through which
an authority supplies its own.

It is for developers of logbook software who want a State's rules to be
supplied to the logbook, not built into it. It is written first for the
[Open Aviation Solutions](https://github.com/Open-Aviation-Solutions) tools:
[pilot-logbook](https://github.com/Open-Aviation-Solutions/pilot-logbook) uses
it, and [au-casa](https://github.com/Open-Aviation-Solutions/au-casa)
implements it for CASA, Australia's authority. The design is recorded in
pilot-logbook's
[ADR 0001](https://github.com/Open-Aviation-Solutions/pilot-logbook/blob/main/ADRs/0001-logbook-shared-kernel.md).

## Status

Not yet released, and not yet written: the repository is set up, and the crate
itself follows in [#1](https://github.com/Open-Aviation-Solutions/logbook-shared-kernel/issues/1).
Its first release is to be `0.1.0`. Until `1.0` a breaking release is a minor
version, so take it by a caret range, as
[`INSTRUCTIONS.md`, "Consumers"](INSTRUCTIONS.md#consumers) sets out.

What it is to hold, in two modules:

- **`entry`**: the logbook's common entry, in ICAO's terms (the date, the
  aircraft or the flight simulation training device, the route, the flight
  rules, the flight time and night, the capacity in which the pilot flew,
  instrument time, approaches, take-offs and landings); an authority's section of it, which
  only that authority reads; and the neutral description of an aircraft type
  that an authority classifies.
- **`authority`**: the trait an authority implements (whether an entry counts
  under its rules, what an aircraft type counts as, its requirements and
  their currency, the questions it asks about an entry and the problems with
  its answers); the machinery its currency checks are written over; and the
  form description, in which it asks those questions as data for the host to
  show.

What it is not to hold:

- Any one State's rules. CASA's are in au-casa; ICAO's terms themselves are
  in [icao-shared-kernel](https://github.com/Open-Aviation-Solutions/icao-shared-kernel),
  which this crate is to depend on.
- Storage: no repositories, databases or file formats. How an entry is
  stored is the application's.
- A user interface. The form description says what to ask, not how to show
  it.

## What it is not

It is not the regulations. Whether a flight meets an authority's requirements
is that authority's implementation to decide, and nothing in this crate
changes a pilot's own responsibility under their State's regulations, such as
CASA's for a personal logbook (reg 61.345).

## Other repositories

- **[icao-shared-kernel](https://github.com/Open-Aviation-Solutions/icao-shared-kernel)**:
  ICAO's terms, which the entry is made of.
- **[au-casa](https://github.com/Open-Aviation-Solutions/au-casa)**: CASA's
  rules (Part 61), and its implementation of this crate's trait.
- **[pilot-logbook](https://github.com/Open-Aviation-Solutions/pilot-logbook)**:
  the logbook, which reads entries through this crate and calls each
  authority through its trait.

## Reporting a problem, and contributing

Report a problem, or propose a change, in this repository's
[issues](https://github.com/Open-Aviation-Solutions/logbook-shared-kernel/issues).

## Development

Requires a Rust toolchain, a C linker (`build-essential` on Debian/Ubuntu) and
`curl`. The first `make check` downloads
[addlicense](https://github.com/google/addlicense)'s release binary into
`.bin/`, and checks it against the checksum pinned in the `Makefile`.

```sh
make help            # list targets
make dev             # build
make test            # run the test suite
make check           # clippy, format check, licence headers, tests
```

## Licence

[Mozilla Public License 2.0](LICENSE). Each source file carries the licence's
notice.
