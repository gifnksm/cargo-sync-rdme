//! <!-- SYNC_RDME_INTEGRATION_TEST::SPAN_START -->
#![cfg_attr(cfg_a, doc = "* CFG A")]
#![cfg_attr(cfg_b, doc = "* CFG B")]
#![cfg_attr(cfg_c, doc = "* CFG C")]
#![cfg_attr(cfg_d, doc = "* CFG D")]
#![cfg_attr(feature = "feat_a", doc = "* FEAT A")]
//! * [`PrivateItem`]
//! <!-- SYNC_RDME_INTEGRATION_TEST::SPAN_END -->

/// This is a private item.
struct PrivateItem;
