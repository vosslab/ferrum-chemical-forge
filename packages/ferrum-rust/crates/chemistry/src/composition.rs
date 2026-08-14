//! Owned isotope-aware molecular composition facts.

use std::cmp::Ordering;

use thiserror::Error;

use crate::AtomicNumber;

/// One isotope-aware elemental identity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CompositionElementKey {
    atomic_number: AtomicNumber,
    isotope: Option<u16>,
}

impl CompositionElementKey {
    /// Construct a validated element/isotope identity.
    #[must_use]
    pub const fn new(atomic_number: AtomicNumber, isotope: Option<u16>) -> Self {
        Self {
            atomic_number,
            isotope,
        }
    }

    /// Return the chemical element.
    #[must_use]
    pub const fn atomic_number(self) -> AtomicNumber {
        self.atomic_number
    }

    /// Return the isotope mass number, when explicitly authored.
    #[must_use]
    pub const fn isotope(self) -> Option<u16> {
        self.isotope
    }

    /// Return the canonical element symbol.
    #[must_use]
    pub fn symbol(self) -> &'static str {
        self.atomic_number.symbol()
    }
}

/// One isotope-aware atom count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ElementCount {
    key: CompositionElementKey,
    count: u64,
}

impl ElementCount {
    pub(crate) const fn new(key: CompositionElementKey, count: u64) -> Self {
        Self { key, count }
    }

    /// Return the isotope-aware element identity.
    #[must_use]
    pub const fn key(self) -> CompositionElementKey {
        self.key
    }

    /// Return the perceived atom count.
    #[must_use]
    pub const fn count(self) -> u64 {
        self.count
    }
}

/// Average-mass contribution and percentage for one isotope-aware element.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElementMassPercentage {
    key: CompositionElementKey,
    average_mass_contribution: f64,
    percentage: f64,
}

/// One checked input entry supplied by a chemistry-engine implementation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoleculeCompositionEntry {
    key: CompositionElementKey,
    atom_count: u64,
    average_mass_contribution: f64,
}

impl MoleculeCompositionEntry {
    /// Construct one positive finite isotope-aware contribution.
    pub fn new(
        key: CompositionElementKey,
        atom_count: u64,
        average_mass_contribution: f64,
    ) -> Result<Self, CompositionBuildError> {
        if atom_count == 0 {
            return Err(CompositionBuildError::InvalidEntry);
        }
        if !average_mass_contribution.is_finite() || average_mass_contribution <= 0.0 {
            return Err(CompositionBuildError::InvalidEntry);
        }
        Ok(Self {
            key,
            atom_count,
            average_mass_contribution,
        })
    }
}

impl ElementMassPercentage {
    pub(crate) const fn new(
        key: CompositionElementKey,
        average_mass_contribution: f64,
        percentage: f64,
    ) -> Self {
        Self {
            key,
            average_mass_contribution,
            percentage,
        }
    }

    /// Return the isotope-aware element identity.
    #[must_use]
    pub const fn key(self) -> CompositionElementKey {
        self.key
    }

    /// Return this entry's contribution to average molecular weight.
    #[must_use]
    pub const fn average_mass_contribution(self) -> f64 {
        self.average_mass_contribution
    }

    /// Return the percentage of the same average-mass contribution total.
    #[must_use]
    pub const fn percentage(self) -> f64 {
        self.percentage
    }
}

/// Complete composition perceived by one authenticated chemistry engine.
#[derive(Clone, Debug, PartialEq)]
pub struct MoleculeComposition {
    formula: String,
    net_formal_charge: i64,
    average_molecular_weight: f64,
    monoisotopic_mass: f64,
    element_counts: Vec<ElementCount>,
    mass_percentages: Vec<ElementMassPercentage>,
}

impl MoleculeComposition {
    pub(crate) fn new(
        formula: String,
        net_formal_charge: i64,
        average_molecular_weight: f64,
        monoisotopic_mass: f64,
        element_counts: Vec<ElementCount>,
        mass_percentages: Vec<ElementMassPercentage>,
    ) -> Self {
        Self {
            formula,
            net_formal_charge,
            average_molecular_weight,
            monoisotopic_mass,
            element_counts,
            mass_percentages,
        }
    }

    /// Build a complete receipt from checked facts supplied by an engine.
    ///
    /// Entries are put into RDKit-compatible Hill/isotope order. Duplicate
    /// isotope keys are rejected rather than silently merged.
    pub fn from_entries(
        net_formal_charge: i64,
        monoisotopic_mass: f64,
        mut entries: Vec<MoleculeCompositionEntry>,
    ) -> Result<Self, CompositionBuildError> {
        if entries.is_empty() {
            return Err(CompositionBuildError::Empty);
        }
        if !monoisotopic_mass.is_finite() || monoisotopic_mass <= 0.0 {
            return Err(CompositionBuildError::InvalidExactMass);
        }
        entries.sort_by(|first, second| hill_order(first.key, second.key));
        if entries.windows(2).any(|pair| pair[0].key == pair[1].key) {
            return Err(CompositionBuildError::DuplicateEntry);
        }
        let mut average_mass = 0.0_f64;
        for entry in &entries {
            average_mass += entry.average_mass_contribution;
            if !average_mass.is_finite() {
                return Err(CompositionBuildError::InvalidAverageMass);
            }
        }
        if average_mass <= 0.0 {
            return Err(CompositionBuildError::InvalidAverageMass);
        }
        let mut counts = Vec::new();
        counts
            .try_reserve_exact(entries.len())
            .map_err(|_| CompositionBuildError::ResourceExhausted)?;
        let mut percentages = Vec::new();
        percentages
            .try_reserve_exact(entries.len())
            .map_err(|_| CompositionBuildError::ResourceExhausted)?;
        for entry in entries {
            counts.push(ElementCount::new(entry.key, entry.atom_count));
            percentages.push(ElementMassPercentage::new(
                entry.key,
                entry.average_mass_contribution,
                entry.average_mass_contribution / average_mass * 100.0,
            ));
        }
        let formula = format_formula(&counts, net_formal_charge, usize::MAX)
            .map_err(|_| CompositionBuildError::ResourceExhausted)?;
        Ok(Self::new(
            formula,
            net_formal_charge,
            average_mass,
            monoisotopic_mass,
            counts,
            percentages,
        ))
    }

    /// Return the RDKit-compatible isotope- and charge-aware formula.
    #[must_use]
    pub fn formula(&self) -> &str {
        &self.formula
    }

    /// Return the checked sum of formal charges.
    #[must_use]
    pub const fn net_formal_charge(&self) -> i64 {
        self.net_formal_charge
    }

    /// Return the average molecular weight in daltons.
    #[must_use]
    pub const fn average_molecular_weight(&self) -> f64 {
        self.average_molecular_weight
    }

    /// Return RDKit's exact molecular mass in daltons.
    #[must_use]
    pub const fn monoisotopic_mass(&self) -> f64 {
        self.monoisotopic_mass
    }

    /// Return isotope-aware counts in RDKit-compatible formula order.
    #[must_use]
    pub fn element_counts(&self) -> &[ElementCount] {
        &self.element_counts
    }

    /// Return average-mass contributions and percentages in the same order.
    #[must_use]
    pub fn mass_percentages(&self) -> &[ElementMassPercentage] {
        &self.mass_percentages
    }

    /// Combine two or more already-authenticated composition receipts.
    ///
    /// Counts, charge, and masses are checked independently. Percentages are
    /// recalculated from the combined average-mass contributions rather than
    /// from displayed or rounded record values.
    pub fn combine(compositions: &[&Self]) -> Result<Self, CompositionAggregationError> {
        if compositions.len() < 2 {
            return Err(CompositionAggregationError::TooFewRecords);
        }
        let mut entry_capacity = 0_usize;
        for composition in compositions {
            if composition.element_counts.len() != composition.mass_percentages.len() {
                return Err(CompositionAggregationError::InvalidRecord);
            }
            entry_capacity = entry_capacity
                .checked_add(composition.element_counts.len())
                .ok_or(CompositionAggregationError::CountOverflow)?;
        }
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(entry_capacity)
            .map_err(|_| CompositionAggregationError::ResourceExhausted)?;
        let mut net_charge = 0_i64;
        let mut exact_mass = 0.0_f64;
        for composition in compositions {
            net_charge = net_charge
                .checked_add(composition.net_formal_charge)
                .ok_or(CompositionAggregationError::ChargeOverflow)?;
            exact_mass += composition.monoisotopic_mass;
            if !exact_mass.is_finite() || exact_mass <= 0.0 {
                return Err(CompositionAggregationError::NonFiniteMass);
            }
            for (count, mass) in composition
                .element_counts
                .iter()
                .zip(&composition.mass_percentages)
            {
                if count.key != mass.key
                    || count.count == 0
                    || !mass.average_mass_contribution.is_finite()
                    || mass.average_mass_contribution <= 0.0
                {
                    return Err(CompositionAggregationError::InvalidRecord);
                }
                entries.push((count.key, count.count, mass.average_mass_contribution));
            }
        }
        entries.sort_by(|first, second| hill_order(first.0, second.0));
        let mut merged: Vec<(CompositionElementKey, u64, f64)> = Vec::new();
        merged
            .try_reserve_exact(entries.len())
            .map_err(|_| CompositionAggregationError::ResourceExhausted)?;
        for (key, count, contribution) in entries {
            if let Some(last) = merged.last_mut().filter(|last| last.0 == key) {
                last.1 = last
                    .1
                    .checked_add(count)
                    .ok_or(CompositionAggregationError::CountOverflow)?;
                last.2 += contribution;
                if !last.2.is_finite() {
                    return Err(CompositionAggregationError::NonFiniteMass);
                }
            } else {
                merged.push((key, count, contribution));
            }
        }
        let mut average_mass = 0.0_f64;
        for entry in &merged {
            average_mass += entry.2;
            if !average_mass.is_finite() {
                return Err(CompositionAggregationError::NonFiniteMass);
            }
        }
        if average_mass <= 0.0 {
            return Err(CompositionAggregationError::NonFiniteMass);
        }
        let mut counts = Vec::new();
        counts
            .try_reserve_exact(merged.len())
            .map_err(|_| CompositionAggregationError::ResourceExhausted)?;
        let mut percentages = Vec::new();
        percentages
            .try_reserve_exact(merged.len())
            .map_err(|_| CompositionAggregationError::ResourceExhausted)?;
        for (key, count, contribution) in merged {
            let percentage = contribution / average_mass * 100.0;
            if !percentage.is_finite() || percentage <= 0.0 {
                return Err(CompositionAggregationError::NonFiniteMass);
            }
            counts.push(ElementCount::new(key, count));
            percentages.push(ElementMassPercentage::new(key, contribution, percentage));
        }
        let formula = format_formula(&counts, net_charge, usize::MAX)
            .map_err(|_| CompositionAggregationError::ResourceExhausted)?;
        Ok(Self::new(
            formula,
            net_charge,
            average_mass,
            exact_mass,
            counts,
            percentages,
        ))
    }
}

/// Failure while constructing one engine-owned composition receipt.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CompositionBuildError {
    /// A molecule composition must contain at least one isotope-aware key.
    #[error("molecule composition requires at least one element entry")]
    Empty,
    /// One entry did not carry a positive count and finite positive contribution.
    #[error("molecule composition entry is not positive and finite")]
    InvalidEntry,
    /// An engine supplied the same isotope-aware key more than once.
    #[error("molecule composition repeats an isotope-aware element entry")]
    DuplicateEntry,
    /// The exact-mass result was not finite and positive.
    #[error("molecule composition exact mass is not finite and positive")]
    InvalidExactMass,
    /// Average-mass contributions did not yield a finite positive total.
    #[error("molecule composition average mass is not finite and positive")]
    InvalidAverageMass,
    /// The owned receipt could not be allocated completely.
    #[error("molecule composition exhausted memory")]
    ResourceExhausted,
}

/// Failure while combining complete composition receipts.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CompositionAggregationError {
    /// Combined selection exists only for two or more molecule records.
    #[error("combined composition requires at least two molecule records")]
    TooFewRecords,
    /// A supposedly complete input receipt violated its internal parallel facts.
    #[error("composition record has inconsistent count or mass facts")]
    InvalidRecord,
    /// An isotope-aware atom count overflowed its public integer domain.
    #[error("combined composition atom count overflows u64")]
    CountOverflow,
    /// The selected molecules' formal charge overflowed its public integer domain.
    #[error("combined composition formal charge overflows i64")]
    ChargeOverflow,
    /// Average or exact mass aggregation ceased to be finite and positive.
    #[error("combined composition mass is not finite and positive")]
    NonFiniteMass,
    /// The owned aggregate could not be allocated completely.
    #[error("combined composition exhausted memory")]
    ResourceExhausted,
}

pub(crate) fn hill_order(first: CompositionElementKey, second: CompositionElementKey) -> Ordering {
    if first.symbol() == "C" {
        return if second.symbol() == "C" {
            first.isotope().cmp(&second.isotope())
        } else {
            Ordering::Less
        };
    }
    if second.symbol() == "C" {
        return Ordering::Greater;
    }
    if first.symbol() == "H" {
        return if second.symbol() == "H" {
            first.isotope().cmp(&second.isotope())
        } else {
            Ordering::Less
        };
    }
    if second.symbol() == "H" {
        return Ordering::Greater;
    }
    first
        .isotope()
        .unwrap_or(0)
        .cmp(&second.isotope().unwrap_or(0))
        .then_with(|| first.symbol().cmp(second.symbol()))
}

pub(crate) fn format_formula(
    entries: &[ElementCount],
    net_charge: i64,
    maximum_bytes: usize,
) -> Result<String, ()> {
    let mut length = 0_usize;
    for entry in entries {
        let key = entry.key();
        length = length
            .checked_add(key.symbol().len())
            .and_then(|value| {
                key.isotope().map_or(Some(value), |isotope| {
                    value.checked_add(decimal_digits(u64::from(isotope)) + 2)
                })
            })
            .and_then(|value| {
                if entry.count() > 1 {
                    value.checked_add(decimal_digits(entry.count()))
                } else {
                    Some(value)
                }
            })
            .ok_or(())?;
    }
    if net_charge != 0 {
        length = length.checked_add(1).ok_or(())?;
        let magnitude = net_charge.unsigned_abs();
        if magnitude > 1 {
            length = length.checked_add(decimal_digits(magnitude)).ok_or(())?;
        }
    }
    if length > maximum_bytes {
        return Err(());
    }
    let mut formula = String::new();
    formula.try_reserve_exact(length).map_err(|_| ())?;
    for entry in entries {
        let key = entry.key();
        if let Some(isotope) = key.isotope() {
            formula.push('[');
            push_decimal(&mut formula, u64::from(isotope));
            formula.push_str(key.symbol());
            formula.push(']');
        } else {
            formula.push_str(key.symbol());
        }
        if entry.count() > 1 {
            push_decimal(&mut formula, entry.count());
        }
    }
    if net_charge != 0 {
        formula.push(if net_charge > 0 { '+' } else { '-' });
        let magnitude = net_charge.unsigned_abs();
        if magnitude > 1 {
            push_decimal(&mut formula, magnitude);
        }
    }
    debug_assert_eq!(formula.len(), length);
    Ok(formula)
}

fn decimal_digits(mut value: u64) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

fn push_decimal(output: &mut String, mut value: u64) {
    let mut digits = [0_u8; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + u8::try_from(value % 10).expect("decimal digit fits u8");
        value /= 10;
        if value == 0 {
            break;
        }
    }
    let text = std::str::from_utf8(&digits[start..]).expect("ASCII digits are UTF-8");
    output.push_str(text);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn composition(
        formula: &str,
        charge: i64,
        exact_mass: f64,
        entries: &[(CompositionElementKey, u64, f64)],
    ) -> MoleculeComposition {
        let average = entries.iter().map(|entry| entry.2).sum::<f64>();
        MoleculeComposition::new(
            formula.to_owned(),
            charge,
            average,
            exact_mass,
            entries
                .iter()
                .map(|entry| ElementCount::new(entry.0, entry.1))
                .collect(),
            entries
                .iter()
                .map(|entry| {
                    ElementMassPercentage::new(entry.0, entry.2, entry.2 / average * 100.0)
                })
                .collect(),
        )
    }

    #[test]
    fn combined_receipt_uses_one_formula_and_mass_basis() {
        let carbon = CompositionElementKey::new(AtomicNumber::try_from(6).expect("carbon"), None);
        let carbon_13 =
            CompositionElementKey::new(AtomicNumber::try_from(6).expect("carbon"), Some(13));
        let hydrogen =
            CompositionElementKey::new(AtomicNumber::try_from(1).expect("hydrogen"), None);
        let first = composition(
            "CH4",
            0,
            16.031,
            &[(carbon, 1, 12.011), (hydrogen, 4, 4.032)],
        );
        let second = composition(
            "[13C]H4+",
            1,
            17.034,
            &[(carbon_13, 1, 13.003), (hydrogen, 4, 4.032)],
        );

        let combined = MoleculeComposition::combine(&[&first, &second]).expect("combined");

        assert_eq!(combined.formula(), "C[13C]H8+");
        assert_eq!(combined.net_formal_charge(), 1);
        assert_eq!(
            combined
                .element_counts()
                .iter()
                .map(|entry| (entry.key(), entry.count()))
                .collect::<Vec<_>>(),
            vec![(carbon, 1), (carbon_13, 1), (hydrogen, 8)]
        );
        assert_eq!(
            combined.average_molecular_weight().to_bits(),
            33.078_f64.to_bits()
        );
        assert_eq!(combined.monoisotopic_mass().to_bits(), 33.065_f64.to_bits());
    }

    #[test]
    fn public_engine_constructor_formats_and_validates_entries() {
        let carbon = CompositionElementKey::new(AtomicNumber::try_from(6).expect("carbon"), None);
        let hydrogen =
            CompositionElementKey::new(AtomicNumber::try_from(1).expect("hydrogen"), None);
        let entries = vec![
            MoleculeCompositionEntry::new(hydrogen, 4, 4.032).expect("hydrogen"),
            MoleculeCompositionEntry::new(carbon, 1, 12.011).expect("carbon"),
        ];

        let composition = MoleculeComposition::from_entries(1, 16.031, entries).expect("receipt");

        assert_eq!(composition.formula(), "CH4+");
        assert_eq!(composition.element_counts()[0].key(), carbon);
        assert_eq!(composition.element_counts()[1].key(), hydrogen);
    }
}
