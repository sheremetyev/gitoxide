//! an index into the pack file
//!
/// From itertools
/// Create an iterator running multiple iterators in lockstep.
///
/// The `izip!` iterator yields elements until any subiterator
/// returns `None`.
///
/// This is a version of the standard ``.zip()`` that's supporting more than
/// two iterators. The iterator element type is a tuple with one element
/// from each of the input iterators. Just like ``.zip()``, the iteration stops
/// when the shortest of the inputs reaches its end.
///
/// **Note:** The result of this macro is in the general case an iterator
/// composed of repeated `.zip()` and a `.map()`; it has an anonymous type.
/// The special cases of one and two arguments produce the equivalent of
/// `$a.into_iter()` and `$a.into_iter().zip($b)` respectively.
///
/// Prefer this macro `izip!()` over [`multizip`] for the performance benefits
/// of using the standard library `.zip()`.
///
/// [`multizip`]: fn.multizip.html
///
/// ```
/// # use itertools::izip;
/// #
/// # fn main() {
///
/// // iterate over three sequences side-by-side
/// let mut results = [0, 0, 0, 0];
/// let inputs = [3, 7, 9, 6];
///
/// for (r, index, input) in izip!(&mut results, 0..10, &inputs) {
///     *r = index * 10 + input;
/// }
///
/// assert_eq!(results, [0 + 3, 10 + 7, 29, 36]);
/// # }
/// ```
macro_rules! izip {
    // @closure creates a tuple-flattening closure for .map() call. usage:
    // @closure partial_pattern => partial_tuple , rest , of , iterators
    // eg. izip!( @closure ((a, b), c) => (a, b, c) , dd , ee )
    ( @closure $p:pat => $tup:expr ) => {
        |$p| $tup
    };

    // The "b" identifier is a different identifier on each recursion level thanks to hygiene.
    ( @closure $p:pat => ( $($tup:tt)* ) , $_iter:expr $( , $tail:expr )* ) => {
        izip!(@closure ($p, b) => ( $($tup)*, b ) $( , $tail )*)
    };

    // unary
    ($first:expr $(,)*) => {
        std::iter::IntoIterator::into_iter($first)
    };

    // binary
    ($first:expr, $second:expr $(,)*) => {
        izip!($first)
            .zip($second)
    };

    // n-ary where n > 2
    ( $first:expr $( , $rest:expr )* $(,)* ) => {
        izip!($first)
            $(
                .zip($rest)
            )*
            .map(
                izip!(@closure a => (a) $( , $rest )*)
            )
    };
}

use filebuffer::FileBuffer;

/// The version of an index file
#[derive(PartialEq, Eq, Ord, PartialOrd, Debug, Hash, Clone, Copy)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[allow(missing_docs)]
pub enum Version {
    V1 = 1,
    V2 = 2,
}

impl Default for Version {
    fn default() -> Self {
        Version::V2
    }
}

impl Version {
    /// The kind of hash to produce to be compatible to this kind of index
    pub fn hash(&self) -> git_hash::Kind {
        match self {
            Version::V1 | Version::V2 => git_hash::Kind::Sha1,
        }
    }
}

const FAN_LEN: usize = 256;

/// A representation of a pack index file
pub struct File {
    data: FileBuffer,
    path: std::path::PathBuf,
    version: Version,
    num_objects: u32,
    fan: [u32; FAN_LEN],
    hash_len: usize,
}

/// Basic file information
impl File {
    /// The version of the pack index
    pub fn version(&self) -> Version {
        self.version
    }
    /// The path of the opened index file
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
    /// The amount of objects stored in the pack and index
    pub fn num_objects(&self) -> u32 {
        self.num_objects
    }
}

const V2_SIGNATURE: &[u8] = b"\xfftOc";
///
pub mod init;

pub(crate) mod access;
pub use access::Entry;

///
pub mod traverse;
mod util;
///
pub mod verify;
///
pub mod write;
