//! Integration test to ensure that specified rustdoc arguments are passed to `cargo rustdoc` as expected.

use rstest::rstest;
use similar_asserts::assert_eq;
use test_helper::{self as helper, Workspace};

#[rstest]
#[case::without_env_flags(
    &[],
    &["CFG A", "FEAT A"],
)]
#[case::with_encoded_rustdocflags(
    &[("CARGO_ENCODED_RUSTDOCFLAGS", "--cfg=cfg_b")],
    &["CFG A", "CFG B", "FEAT A"],
)]
#[case::with_rustdocflags(
    &[("RUSTDOCFLAGS", "--cfg=cfg_c")],
    &["CFG A", "CFG C", "FEAT A"],
)]
#[case::encoded_rustdocflags_take_precedence_over_rustdocflags(
    &[("CARGO_ENCODED_RUSTDOCFLAGS", "--cfg=cfg_b"), ("RUSTDOCFLAGS", "--cfg=cfg_c")],
    &["CFG A", "CFG B", "FEAT A"],
)]
#[case::empty_encoded_rustdocflags_mean_no_additional_env_flags(
    &[("CARGO_ENCODED_RUSTDOCFLAGS", "")],
    &["CFG A", "FEAT A"],
)]
#[case::empty_encoded_rustdocflags_still_take_precedence_over_rustdocflags(
    &[("CARGO_ENCODED_RUSTDOCFLAGS", ""), ("RUSTDOCFLAGS", "--cfg=cfg_c")],
    &["CFG A", "FEAT A"],
)]
#[case::rustdocflags_ignore_extra_whitespace(
    &[("RUSTDOCFLAGS", "  --cfg=cfg_c  --cfg=cfg_d  ")],
    &["CFG A", "CFG C", "CFG D", "FEAT A"],
)]
fn rustdoc_flags(#[case] envs: &[(&str, &str)], #[case] expected: &[&str]) {
    let crate_name = "rustdoc_args";
    let workspace = Workspace::from_fixture(crate_name);
    let readme_path = workspace
        .metadata()
        .root_package()
        .unwrap()
        .readme()
        .unwrap();
    workspace
        .cargo_sync_rdme_default()
        .envs(envs.iter().copied())
        .assert()
        .success();

    let list_items = helper::collect_list_item_from_markdown_file(&readme_path);
    assert_eq!(list_items, expected);

    let md_links = helper::collect_links_from_markdown_file(&readme_path, crate_name);
    assert_eq!(
        md_links,
        [(
            "struct.PrivateItem.html".to_owned(),
            "struct rustdoc_args::PrivateItem".to_owned(),
        )]
    );
}
