#pragma once
#include <glib.h>
/* Local-only validation: no mounted file body or network operation is read. */
gboolean dn_scope_contains_path(const gchar *path);
