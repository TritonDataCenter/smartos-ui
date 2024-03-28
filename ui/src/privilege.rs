/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

//! When running on an illumos host (which is the primary use-case of this
//! application) the UI process does not execute any programs, only needs to
//! read the TLS certificate and key before starting the HTTP server, and does
//! not need to write to any file other than it's log file.
//!
//! When **not** running on an illumos system, it is assumed the process is
//! running for development and testing purposes (e.g. on macOS) in which case
//! no privileges are dropped.

use slog::Logger;

#[cfg(not(target_os = "illumos"))]
use slog::warn;

#[cfg(target_os = "illumos")]
use slog::info;

#[cfg(target_os = "illumos")]
use illumos_priv::{setppriv, PrivOp, PrivPtype, PrivSet, Privilege};

#[cfg(target_os = "illumos")]
use privdrop::PrivDrop;

/// Drop privileges on an illumos system.
/// The UI process will be started as the "root" user, chroot into an empty
/// directory, change to the "nobody" user, and delete any unneeded privileges
/// from its privilege set.
#[cfg(target_os = "illumos")]
pub fn drop_privileges(log: &Logger, chroot_dir: &str) {
    info!(log, "Running on illumos, dropping privileges");
    PrivDrop::default()
        .chroot(chroot_dir)
        .user("nobody")
        .apply()
        .unwrap_or_else(|e| {
            panic!(
                "Failed to chroot to {} and change to user nobody: {}",
                chroot_dir, e
            )
        });

    // expect() is used here as this code runs before starting the HTTP server
    let err = "Failed to set new illumos Privilege set";
    let mut privileges = PrivSet::new_basic().expect(err);
    privileges.delset(Privilege::ProcExec).expect(err);
    privileges.delset(Privilege::ProcFork).expect(err);

    setppriv(PrivOp::Set, PrivPtype::Permitted, &privileges).expect(err);
    setppriv(PrivOp::Set, PrivPtype::Effective, &privileges).expect(err);
}

#[cfg(not(target_os = "illumos"))]
pub fn drop_privileges(log: &Logger, _: &str) {
    warn!(log, "Not running on illumos, not dropping privileges");
}
