//! Le rendu au propre de Fanoratana.
//!
//! - [`scene`] : une page devient une liste de primitives ([`scene::Prim`]), dessinées pareil à
//!   l'écran et à l'export.
//! - [`maths`] : un petit moteur de formules (sous-ensemble de LaTeX).
//! - [`rough`] : le trait fait main des formes, façon Excalidraw.
//! - [`export`] : SVG, PNG et PDF.
//! - [`polices`] : Caveat, Noto Sans Math, Nunito, embarquées.

pub mod export;
pub mod expr;
pub mod maths;
pub mod polices;
pub mod rough;
pub mod scene;

pub use scene::Prim;
