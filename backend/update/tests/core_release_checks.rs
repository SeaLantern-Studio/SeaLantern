#![allow(dead_code)]
#![allow(unused_imports)]

#[path = "../src/constants.rs"]
pub mod constants;

#[path = "../src/types.rs"]
pub mod types;

#[path = "../src/checksum.rs"]
mod checksum;

#[path = "../src/download.rs"]
mod download;

#[path = "../src/github.rs"]
mod github;

#[path = "../src/install_support.rs"]
mod install_support;

#[path = "../src/pending.rs"]
mod pending;

#[path = "../src/version.rs"]
mod version;

#[path = "../src/asset_selector.rs"]
mod asset_selector;

#[path = "../src/cnb.rs"]
mod cnb;
