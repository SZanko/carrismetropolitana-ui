#[cfg(feature = "std")]
mod client_std;

#[cfg(feature = "embedded")]
mod client_embedded;

#[cfg(feature = "std")]
pub use client_std::CarrisClient;

#[cfg(feature = "embedded")]
pub use client_embedded::CarrisClient;