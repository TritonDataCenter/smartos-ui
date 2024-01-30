use slog::Logger;

#[cfg(not(target_os = "illumos"))]
use slog::warn;

#[cfg(target_os = "illumos")]
use slog::info;

#[cfg(target_os = "illumos")]
use illumos_priv::{setppriv, PrivOp, PrivPtype, PrivSet, Privilege};

#[cfg(target_os = "illumos")]
use privdrop::PrivDrop;

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
