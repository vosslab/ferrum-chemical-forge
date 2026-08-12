//! Isolated import support for one historical compact carbohydrate notation.
//!
//! The syntax in [`legacy_compact_v1`] is a bounded migration format, not a
//! Ferrum carbohydrate interchange format or core sugar model. It neither
//! creates molecules nor infers a sugar from a molecule; those remain separate
//! codec and depiction capabilities.

pub mod legacy_compact_v1;
mod semantic;
mod syntax;

#[cfg(test)]
mod tests;
