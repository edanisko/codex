//! Content preferences shared by streaming and completed Markdown renderers.
//!
//! Seed before startup previews, then refresh when resolved session settings become active.
//! Tests isolate preferences per thread so parallel renderer tests cannot affect one another.

use codex_config::types::TuiRendering;

#[cfg(not(test))]
static WORKSPACE_IS_LOCAL: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
thread_local! {
    static WORKSPACE_IS_LOCAL: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Remote execution paths must never be resolved against the frontend's filesystem.
pub(crate) fn set_workspace_is_local(is_local: bool) {
    #[cfg(not(test))]
    WORKSPACE_IS_LOCAL.store(is_local, std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    WORKSPACE_IS_LOCAL.set(is_local);
}

pub(crate) fn local_file_links_enabled() -> bool {
    #[cfg(not(test))]
    let is_local = WORKSPACE_IS_LOCAL.load(std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    let is_local = WORKSPACE_IS_LOCAL.get();
    is_local && current().local_file_links
}

#[cfg(not(test))]
static RENDERING: std::sync::LazyLock<std::sync::RwLock<TuiRendering>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(TuiRendering::default()));

#[cfg(test)]
thread_local! {
    static RENDERING: std::cell::Cell<TuiRendering> = std::cell::Cell::new(TuiRendering::default());
}

pub(crate) fn init(rendering: TuiRendering) {
    #[cfg(not(test))]
    {
        *RENDERING
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = rendering;
    }
    #[cfg(test)]
    RENDERING.set(rendering);
}

pub(crate) fn current() -> TuiRendering {
    #[cfg(not(test))]
    {
        *RENDERING
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    #[cfg(test)]
    {
        RENDERING.get()
    }
}

#[cfg(test)]
#[path = "preferences_tests.rs"]
mod tests;
