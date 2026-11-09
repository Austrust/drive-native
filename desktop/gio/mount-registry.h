#pragma once
#include <glib.h>
/* Local-only validation: no mounted file body or network operation is read. */
gboolean dn_scope_contains_path(const gchar *path);
/* Prefix-cache counters: repeated paths under one verified mount root reuse the
 * validated root instead of rescanning the registry directory. */
typedef struct {
    guint64 checks;
    guint64 cache_hits;
    guint64 scan_misses;
    guint64 cached_roots;
} DnScopeCacheStats;
void dn_scope_cache_stats(DnScopeCacheStats *stats);
