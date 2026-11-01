# Directory view reuse during initialization

Directory opens now compare the committed status journal epoch/sequence and
reset/overflow counters under the same Store guard used to select the view.
Private initialization receipts and pure inode allocation no longer invalidate
an otherwise unchanged shared directory view. Namespace and ACL commits still
invalidate conservatively; existing directory handles retain stable cookies.

The cache remains weak: this improves reuse while another handle holds a view,
not an indefinite cache across every close/reopen. New opens still take the Store
lock. This does not solve all large-directory latency or revocation through an
already open handle. Future standalone inode remapping must invalidate the stamp.

Isolated local validation on 2026-11-01 passed 235 core and 12 filesystem library
tests, plus the workspace all-feature/all-target check. One core and four kernel
tests were ignored by their default harness; no new kernel or NAS performance
result is claimed. Cases cover candidate receipts, concurrent inode allocation,
semantic changes, epoch replacement, rollback and journal reset/overflow.

This source experiment follows frozen development package `.5`; it is not part
of that package. See [the full delivery gates](../DELIVERY_GATES.md).
