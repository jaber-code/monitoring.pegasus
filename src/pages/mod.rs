//! One module per screen. Each exports a single `#[component]` used in the route
//! table in [`crate::app`].

mod current_jobs;
mod home;
mod not_found;

pub use current_jobs::CurrentJobsPage;
pub use home::HomePage;
pub use not_found::NotFound;
