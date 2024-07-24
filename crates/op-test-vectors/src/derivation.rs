//! Module containing the derivation test fixture.

use serde::{Deserialize, Serialize};

/// The derivation fixture is the top-level object that contains
/// everything needed to run a derivation test.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DerivationFixture {}
