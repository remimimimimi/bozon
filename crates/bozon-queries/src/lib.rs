//! All kind of compiler queries upon which it is built.
//!
//! # Why queries?
//!
//! The programming language is no longer equal to the compiler. It needs
//! appropriate tools, and to support development makes sense to use a
//! different architecture rather than a normal pipeline-based. This
//! architecture is called query-based which is used in the bozon compiler.
//! The key idea of query-based compiler architecture is that you define
//! a compiler as a set of queries, which are just functions, in
//! mathematical definition, that take some keys and produce some values.
//! Thus compiler API becomes more declarative and easier to use.
//!
//! * Great for tooling creators. Queries reduce the cognitive load on the
//! creators of the toolchain dramatically because they are pure functions,
//! so creators don’t need to think and worry about call order or temporal
//! effects.
//!
//! * Easy to parallelize. Query-based compilers allow users to query
//! without worrying about the right call time and synchronization of
//! states. Also, queries are memoized and will be executed only at the
//! first call.
//!
//! * Ergonomic compiler architecture. With queries compiler writer
//! doesn't have to handle an update to and invalidation of a bunch of ad-hoc
//! caches, which can be the result when adding incremental updates to a
//! traditional compiler pipeline.
//!
//! # How to read this documentation?
//!
//! This documentation hides all noisy things generated by salsa's macroses.
//! So it contains only necessary components and can be easily discovered
//! by just looking for corresponding traits in the [`groups`] module.

//!
//! # Examples
//!
//! For example let's create some simple tool using queries. TODO: when
//! compiler will be more mature.

/// Definition of all queries traits.
pub mod groups;
/// Database interner structure
pub(crate) mod interner;
/// Internable values
pub(crate) mod values;

/// Repository of all cache.
#[salsa::database(groups::VfsStorage, groups::LanguageStorage, groups::ParserStorage)]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Self>,
}

impl<'a> salsa::Database for Database {}
