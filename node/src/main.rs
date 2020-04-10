//! Certifybook Chain Node CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;

fn main() -> sc_cli::Result<()> {
	let version = sc_cli::VersionInfo {
		name: "Certifybook Node",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "certifybook-chain",
		author: "Anonymous",
		description: "Certifybook Node",
		support_url: "support.anonymous.an",
		copyright_start_year: 2017,
	};

	command::run(version)
}
