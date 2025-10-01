//! BGI Stroked Font converted from [SDL_bgi](https://sdl-bgi.sourceforge.io/).

#![forbid(unsafe_code)]

#[cfg(feature = "bold")]
pub mod bold;

#[cfg(feature = "euro")]
pub mod euro;

#[cfg(feature = "goth")]
pub mod goth;

#[cfg(feature = "lcom")]
pub mod lcom;

#[cfg(feature = "litt")]
pub mod litt;

#[cfg(feature = "sans")]
pub mod sans;

#[cfg(feature = "scri")]
pub mod scri;

#[cfg(feature = "simp")]
pub mod simp;

#[cfg(feature = "trip")]
pub mod trip;

#[cfg(feature = "tscr")]
pub mod tscr;
