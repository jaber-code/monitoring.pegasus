//! App chrome: the persistent sidebar + topbar, and the [`AppShell`] that wraps
//! every screen's content between them.

mod app_shell;
mod sidebar;
mod topbar;

pub use app_shell::AppShell;
pub use sidebar::{Sidebar, SidebarItem};
pub use topbar::Topbar;
