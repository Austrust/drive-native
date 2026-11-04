# Metadata cache policy: names always revalidate

This development integration separates positive-entry and attribute lifetimes.
It supersedes the positive-name caching contract of `de19ae0`; the earlier
approximately four-second static scan result is not a performance claim for this
policy. The corrected functional and package-build regressions have passed;
[validation and remaining gates](validation/fs-recovery-001/REPORT.md) distinguish
those results from pending installation and performance acceptance.

## Published replies

- LOOKUP and READDIRPLUS always publish **entry TTL 0**, including when
  `DRIVE_NATIVE_FS_TTL_MS` is set. Negative entries remain uncached.
- The environment value controls only clean authoritative **attribute TTL**
  (default 1000 ms). Dirty/drafted/unlinked objects, attributes adjusted from an
  open handle, and candidate previews use attribute TTL 0.
- A notifier must be installed before attributes become cacheable. Runtime
  feeder overflow/disconnection and hub delivery failures stop new leases.
  This does not recall an already prepared reply. Installed or in-flight
  attribute replies remain subject to their TTL and kernel attribute-version
  checks; a failure flag does not prove immediate attribute revocation.
- Existing open descriptors preserve their POSIX identity/content semantics.
  New path-based opens must revalidate the name; checking a cached inode in
  Store cannot prove that the caller's original path still names that inode.

The configured TTL is therefore not a switch for positive name caching.
`INC_EPOCH` support and directory quieting are not substitutes for this rule.
Linux positive dentry revalidation and READDIRPLUS do not share one universally
protected reply-install boundary across the supported kernel versions.

## Preview and notification boundaries

Candidate names are revocable metadata even when a PLUS entry has nodeid 0.
Preview PLUS validates its current stamp while holding Store until one bounded
reply has been sent. A busy Store never authorizes an old preview view. Warm
authority continuation retains its existing dirent-only fallback.

Every runtime notification producer uses the same non-blocking feeder. A full
or disconnected upstream channel disables new attribute leases through the
mount hub; loss can no longer remain hidden before the hub's own queue. Neither
producer waits for a kernel invalidation while holding Store. Notification
failure and outstanding lease expiry remain distinct events.

## Validation coverage

- [Preview boundary unit tests](../crates/drive-fs/src/preview_publication_tests.rs):
  busy Store plus child denial, guard held through send, promotion revokes old view.
- [Kernel preview PLUS regression](../crates/drive-fs/tests/preview_plus_revoke.rs):
  require actual PLUS diagnostics and reject old names after a denied child while
  Store is busy. Run explicitly with `DRIVE_NATIVE_FS_DIAGNOSTICS=1`.
- [New-open path regressions](../crates/drive-fs/tests/entry_revalidation.rs):
  keep an existing descriptor, move its inode within or across directories,
  and verify a new old-path `O_TRUNC` fails without touching the moved content.
  No notification is sent; a 30-second attribute setting makes natural cache
  expiry distinguishable from required name revalidation.
- [Feeder unit tests](../crates/drive-runtime/src/notification_queue_tests.rs):
  queue full/disconnected disables leases and exposes one persistent failure.

The explicit kernel regressions passed in snapshot 002 and again in the frozen
package014 debug/release checks. The first run had exposed reversed ordinary
entry TTL arguments; unequal-TTL ABI checks now cover that boundary. The
30-second path tests set their own attribute TTL in both environment matrices,
so they are not a zero-versus-positive attribute-cache comparison.

Original FSearch behavior passed on a separately built diagnostic fixture;
ordinary-ELF timing, installation, Mint/NAS acceptance and the broader permission
policy remain separate gates. Passing these regressions does not establish
strict instantaneous attribute invalidation on every supported kernel.

Kernel references: [Linux 6.14 dentry revalidation](https://github.com/torvalds/linux/blob/v6.14/fs/fuse/dir.c),
[Linux 6.18 PLUS linking](https://github.com/torvalds/linux/blob/v6.18/fs/fuse/readdir.c).
