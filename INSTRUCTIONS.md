# logbook-shared-kernel — Working Instructions

The logbook's shared kernel, in Rust: the part of a pilot's logbook that
[pilot-logbook](https://github.com/Open-Aviation-Solutions/pilot-logbook) and
each licensing authority's rules (CASA's, in
[au-casa](https://github.com/Open-Aviation-Solutions/au-casa)) agree on. It is
decided in pilot-logbook's
[ADR 0001](https://github.com/Open-Aviation-Solutions/pilot-logbook/blob/main/ADRs/0001-logbook-shared-kernel.md);
read it before changing what the crate holds.

## Purpose and scope

The crate is to hold, in one crate and two modules (split only if something
wants the records without the interface):

- **`entry`**: the logbook's common entry, the pilot's claim in ICAO's terms
  (taken from `icao-shared-kernel`); an authority's section of it, stored as
  the authority gives it and read only by that authority; and the neutral
  description of an aircraft type.
- **`authority`**: the trait an authority implements to supply its rules; the
  accumulator and check traits, the event window and `CurrencyResult`, so
  each authority writes its currency checks over the same machinery; and the
  form description, in which an authority asks its questions about an entry
  as data, with no dependence on a user interface.

Neither is written yet: the repository is set up first, and the crate follows
in [#1](https://github.com/Open-Aviation-Solutions/logbook-shared-kernel/issues/1),
from a plan agreed on that issue. `src/lib.rs` holds only the crate's doc
comment until then.

Out of scope (do not add here): any one State's rules, names or regulations
(CASA's are au-casa's), ICAO's terms themselves (`icao-shared-kernel`'s),
storage and stored formats (the application's), and any user interface.

## Conventions

- **Discuss before implementing** non-trivial design changes.
- **Regulatory terms only**: names and terminology must be verifiable against
  ICAO's Annex 1 for the common entry, and against the regulation an example or
  test quotes. A name that no regulation uses is not introduced.
- **The MPL notice heads each file** in `src/`, `tests/`, the `Makefile`,
  `Cargo.toml` and the workflows (the licence's Exhibit A, in that file's
  comment syntax); `addlicense -l mpl <file>` adds it. `make license-check`,
  part of `make check`, runs `addlicense -check` over the tracked files, which
  fails on a Rust, TOML or YAML file with no licence header, though it does not
  check that the header is the MPL's wording. It does not check the
  `Makefile`, which it does not recognise.

## Consumers

`pilot-logbook` is to depend on this crate, and `au-casa` to implement its
trait.

Choose the best API, not the compatible one: while the crate is on `0.x`, a
breaking change is fine, landed with a matching PR in each consumer.

While on `0.x`, a breaking release bumps the minor version (`0.2.0`), and a
compatible one the patch. Breaking includes a change to the API and any change
that could alter what a consumer derives from the crate. Consumers take the
crate by a caret range (`^0.1.0`), not pinned exactly, so that a build with
several of them (au-casa, its sibling country crates and a logbook using them)
has one copy of the crate and its types are the same in each; a cargo update
then takes up patches but not a breaking release, which each consumer moves to
in a pull request of its own (decided on
[au-casa#24](https://github.com/Open-Aviation-Solutions/au-casa/issues/24)).

## Releasing

Bump `version` in `Cargo.toml` in a pull request, and merge it: that is the
release. On each push to `main`, the Release workflow
(`.github/workflows/release.yml`) asks crates.io whether `Cargo.toml`'s version
is there yet. When it is not, it runs `make check`, publishes it through trusted
publishing (no crates.io token is kept in the repository), and tags the commit
`v<version>`. Don't push version tags by hand: nothing runs on them.

Only `main` can release. The publishing job runs in the `release` environment,
which GitHub allows only on `main`, and crates.io's trusted publisher accepts
only that environment; `main` changes only through a pull request with CI
passing.

### Setting it up

crates.io offers trusted publishing only for a crate that already exists, so
the first release is a person's, in this order:

1. **Publish the first version by hand**: `cargo publish` from `main`, once the
   crate itself is written, so that the first version is the real `0.1.0` and
   not a placeholder.
2. **Set up GitHub**: create the `release` environment with a deployment rule
   allowing only the branch `main`, and protect `main` as
   `icao-shared-kernel`'s is: a pull request, the `check` status required,
   admins included.
3. **Register the trusted publisher on crates.io**: the crate's Settings →
   Trusted Publishing, owner `Open-Aviation-Solutions`, repository
   `logbook-shared-kernel`, workflow `release.yml`, environment `release`.

Until steps 1 and 3 are done, the Release run goes red on each push to `main`:
it sees the version missing from crates.io and tries, and the
trusted-publishing exchange fails, so nothing is published. Until step 2 is
done, GitHub creates the `release` environment with no rule on it.

## Commands

```sh
make test       # cargo test — the test suite
make lint       # cargo clippy -- -D warnings
make fmt        # cargo fmt
make check      # lint + fmt check + licence headers + test
```
