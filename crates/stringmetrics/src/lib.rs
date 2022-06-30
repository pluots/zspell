//! # Stringmetrics library
//!
//! A library for applying text- and token- based comparison algorithms to
//! determine the similarity of two strings or sets. The core module is
//! [`algorithms`], which contains algorithms for determining closeness; more
//! are on the way.
//!
//! Note that this module is very much a work in progress, and it is likely that
//! interfaces may change. Large parts of this module are still under
//! construction. The eventual goal is to implement many of the metrics listed
//! on [this wikipedia page](https://en.wikipedia.org/wiki/String_metric),
//! generalized to any hashable datatype.

pub mod algorithms;
pub mod spellcheck;
pub mod tokenizers;
