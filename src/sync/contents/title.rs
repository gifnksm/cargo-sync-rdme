use cargo_metadata::Package;

pub(super) fn create(package: &Package) -> String {
    format!("# {}\n", package.name)
}
