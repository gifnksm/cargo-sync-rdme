use crate::sync::PackageSyncContext;

pub(super) fn create(cx: &PackageSyncContext<'_>) -> String {
    format!("# {}\n", cx.package.name)
}
