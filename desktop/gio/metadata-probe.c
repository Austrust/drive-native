/* Isolated research module: GIO metadata queries without content sniffing.
 * Not installed or enabled for normal desktop processes. */
#include <gio/gio.h>
#include <string.h>
#include "mount-registry.h"

typedef struct { GObject parent; GFile *inner; } ProbeFile;
typedef struct { GObjectClass parent; } ProbeFileClass;
typedef struct { GFileEnumerator parent; GFileEnumerator *inner; gboolean mime; } ProbeEnumerator;
typedef struct { GFileEnumeratorClass parent; } ProbeEnumeratorClass;
typedef struct { GFileMonitor parent; GFileMonitor *inner; gulong changed; } ProbeMonitor;
typedef struct { GFileMonitorClass parent; } ProbeMonitorClass;
static void file_iface_init(GFileIface *iface);
G_DEFINE_TYPE_WITH_CODE(ProbeFile, probe_file, G_TYPE_OBJECT,
                       G_IMPLEMENT_INTERFACE(G_TYPE_FILE, file_iface_init))
G_DEFINE_TYPE(ProbeEnumerator, probe_enumerator, G_TYPE_FILE_ENUMERATOR)
G_DEFINE_TYPE(ProbeMonitor, probe_monitor, G_TYPE_FILE_MONITOR)
static GVfsClass *base_vfs_class;

static GFile *unwrap(GFile *f) {
    return G_TYPE_CHECK_INSTANCE_TYPE(f, probe_file_get_type()) ? ((ProbeFile *)f)->inner : f;
}
static GFile *wrap(GFile *inner) {
    if (!inner) return NULL;
    if (G_TYPE_CHECK_INSTANCE_TYPE(inner, probe_file_get_type())) return inner;
    gchar *path = g_file_get_path(inner);
    gboolean in_root = dn_scope_contains_path(path);
    g_free(path);
    if (!in_root) return inner;
    ProbeFile *f = g_object_new(probe_file_get_type(), NULL);
    f->inner = inner;
    return G_FILE(f);
}
static void probe_file_init(ProbeFile *f) { f->inner = NULL; }
static void file_finalize(GObject *o) {
    g_clear_object(&((ProbeFile *)o)->inner);
    G_OBJECT_CLASS(probe_file_parent_class)->finalize(o);
}
static void probe_file_class_init(ProbeFileClass *c) { G_OBJECT_CLASS(c)->finalize = file_finalize; }

/* Expand the standard namespace explicitly so GLocalFile never sees content-type.
 * Keep all requested non-standard namespaces. Future attributes need review. */
static const gchar *safe_standard =
    "standard::type,standard::is-hidden,standard::is-backup,standard::is-symlink,"
    "standard::is-virtual,standard::is-volatile,standard::name,standard::display-name,"
    "standard::edit-name,standard::copy-name,"
    "standard::fast-content-type,standard::size,standard::allocated-size,"
    "standard::symlink-target,standard::target-uri,standard::sort-order";
static gchar *safe_attributes(const gchar *attrs, gboolean *mime) {
    GFileAttributeMatcher *matcher = g_file_attribute_matcher_new(attrs);
    *mime = g_file_attribute_matcher_matches(matcher, G_FILE_ATTRIBUTE_STANDARD_CONTENT_TYPE);
    g_file_attribute_matcher_unref(matcher);
    GString *result = g_string_new(safe_standard);
    gchar **parts = g_strsplit(attrs, ",", -1);
    for (gchar **p = parts; *p; p++) {
        if (!strcmp(*p, "*")) {
            g_string_append(result, ",id::*,access::*,mountable::*,time::*,unix::*,dos::*,owner::*,"
                "thumbnail::*,filesystem::*,gvfs::*,selinux::*,trash::*,metadata::*,xattr::*,xattr-sys::*");
        } else if (strcmp(*p, "standard::*") && strcmp(*p, G_FILE_ATTRIBUTE_STANDARD_CONTENT_TYPE)
                   && strcmp(*p, G_FILE_ATTRIBUTE_STANDARD_ICON) && strcmp(*p, G_FILE_ATTRIBUTE_STANDARD_SYMBOLIC_ICON)) {
            g_string_append_c(result, ','); g_string_append(result, *p);
        }
    }
    g_strfreev(parts);
    return g_string_free(result, FALSE);
}
static GFileInfo *finish_info(GFileInfo *info, gboolean mime) {
    if (info) {
        const gchar *type = g_file_info_get_attribute_string(info, G_FILE_ATTRIBUTE_STANDARD_FAST_CONTENT_TYPE);
        if (!type) type = "application/octet-stream";
        if (mime) g_file_info_set_content_type(info, type);
        GIcon *icon = g_content_type_get_icon(type);
        g_file_info_set_icon(info, icon); g_object_unref(icon);
        icon = g_content_type_get_symbolic_icon(type);
        g_file_info_set_symbolic_icon(info, icon); g_object_unref(icon);
    }
    return info;
}
static gboolean managed_file(GFile *f) {
    gchar *path=g_file_get_path(unwrap(f));
    gboolean managed=dn_scope_contains_path(path);g_free(path);return managed;
}
static GFileInfo *query_info(GFile *f, const gchar *attrs, GFileQueryInfoFlags flags,
                            GCancellable *c, GError **error) {
    if (!managed_file(f)) return g_file_query_info(unwrap(f),attrs,flags,c,error);
    gboolean mime; gchar *safe = safe_attributes(attrs, &mime);
    GFileInfo *info = g_file_query_info(unwrap(f), safe, flags, c, error);
    g_free(safe); return finish_info(info, mime);
}
static GFileInfo *query_fs(GFile *f, const gchar *attrs, GCancellable *c, GError **error) {
    GFileInfo *info = g_file_query_filesystem_info(unwrap(f), attrs, c, error);
    if (info && managed_file(f)) g_file_info_set_attribute_uint32(info, G_FILE_ATTRIBUTE_FILESYSTEM_USE_PREVIEW,
                                              G_FILESYSTEM_PREVIEW_TYPE_NEVER);
    return info;
}
static GFileInfo *enum_next(GFileEnumerator *e, GCancellable *c, GError **error) {
    ProbeEnumerator *self = (ProbeEnumerator *)e;
    return finish_info(g_file_enumerator_next_file(self->inner, c, error), self->mime);
}
static gboolean enum_close(GFileEnumerator *e, GCancellable *c, GError **error) {
    return g_file_enumerator_close(((ProbeEnumerator *)e)->inner, c, error);
}
static void enum_finalize(GObject *o) {
    /* The base finalizer closes an unfinished enumeration through close_fn.
     * Close while the wrapped enumerator is still alive, then release it. */
    if (!g_file_enumerator_is_closed(G_FILE_ENUMERATOR(o)))
        g_file_enumerator_close(G_FILE_ENUMERATOR(o), NULL, NULL);
    g_clear_object(&((ProbeEnumerator *)o)->inner);
    G_OBJECT_CLASS(probe_enumerator_parent_class)->finalize(o);
}
static void probe_enumerator_class_init(ProbeEnumeratorClass *c) {
    G_OBJECT_CLASS(c)->finalize = enum_finalize;
    G_FILE_ENUMERATOR_CLASS(c)->next_file = enum_next;
    G_FILE_ENUMERATOR_CLASS(c)->close_fn = enum_close;
}
static void probe_enumerator_init(ProbeEnumerator *e) { e->inner = NULL; }
static GFileEnumerator *enumerate(GFile *f, const gchar *attrs, GFileQueryInfoFlags flags,
                                 GCancellable *c, GError **error) {
    if (!managed_file(f)) return g_file_enumerate_children(unwrap(f),attrs,flags,c,error);
    gboolean mime; gchar *safe = safe_attributes(attrs, &mime);
    GFileEnumerator *inner = g_file_enumerate_children(unwrap(f), safe, flags, c, error);
    g_free(safe); if (!inner) return NULL;
    ProbeEnumerator *e = g_object_new(probe_enumerator_get_type(), "container", f, NULL);
    e->inner = inner; e->mime = mime; return G_FILE_ENUMERATOR(e);
}
static GFile *dup_file(GFile *f) { return wrap(g_file_dup(unwrap(f))); }
static guint hash_file(GFile *f) { return g_file_hash(unwrap(f)); }
static gboolean equal_file(GFile *a, GFile *b) { return g_file_equal(unwrap(a), unwrap(b)); }
static gboolean native(GFile *f) { return g_file_is_native(unwrap(f)); }
static gboolean has_scheme(GFile *f, const gchar *s) { return g_file_has_uri_scheme(unwrap(f), s); }
#define STRING_GETTER(name) static gchar *name(GFile *f) { return g_file_##name(unwrap(f)); }
STRING_GETTER(get_uri_scheme)
STRING_GETTER(get_basename)
STRING_GETTER(get_path)
STRING_GETTER(get_uri)
STRING_GETTER(get_parse_name)
static GFile *get_parent(GFile *f) { return wrap(g_file_get_parent(unwrap(f))); }
static gboolean prefix(GFile *a, GFile *b) { return g_file_has_prefix(unwrap(b), unwrap(a)); }
static gchar *relative(GFile *a, GFile *b) { return g_file_get_relative_path(unwrap(a), unwrap(b)); }
static GFile *resolve(GFile *f, const gchar *path) { return wrap(g_file_resolve_relative_path(unwrap(f), path)); }
static GFile *display_child(GFile *f, const gchar *name, GError **error) {
    return wrap(g_file_get_child_for_display_name(unwrap(f), name, error));
}
static GFileInputStream *read_file(GFile *f, GCancellable *c, GError **e) { return g_file_read(unwrap(f), c, e); }
/* Monitor events must keep the same metadata contract as enumerated files.
 * Otherwise Nemo queries the unwrapped newly created/renamed GFile and sniffs it. */
static void monitor_changed(GFileMonitor *inner, GFile *file, GFile *other,
                            GFileMonitorEvent event, gpointer data) {
    (void)inner;
    ProbeMonitor *self = data;
    if (g_file_monitor_is_cancelled(G_FILE_MONITOR(self))) return;
    GFile *wrapped_file = file ? wrap(g_object_ref(file)) : NULL;
    GFile *wrapped_other = other ? wrap(g_object_ref(other)) : NULL;
    g_file_monitor_emit_event(G_FILE_MONITOR(self), wrapped_file, wrapped_other, event);
    g_clear_object(&wrapped_file); g_clear_object(&wrapped_other);
}
static gboolean monitor_cancel(GFileMonitor *monitor) {
    ProbeMonitor *self = (ProbeMonitor *)monitor;
    if (!self->inner) return TRUE;
    if (self->changed) {
        g_signal_handler_disconnect(self->inner, self->changed);
        self->changed = 0;
    }
    return g_file_monitor_cancel(self->inner);
}
static void monitor_dispose(GObject *object) {
    monitor_cancel(G_FILE_MONITOR(object));
    g_clear_object(&((ProbeMonitor *)object)->inner);
    G_OBJECT_CLASS(probe_monitor_parent_class)->dispose(object);
}
static void probe_monitor_class_init(ProbeMonitorClass *klass) {
    G_OBJECT_CLASS(klass)->dispose = monitor_dispose;
    G_FILE_MONITOR_CLASS(klass)->cancel = monitor_cancel;
}
static void probe_monitor_init(ProbeMonitor *self) { self->inner = NULL; self->changed = 0; }
static GFileMonitor *wrap_monitor(GFileMonitor *inner) {
    if (!inner) return NULL;
    ProbeMonitor *self = g_object_new(probe_monitor_get_type(), NULL);
    self->inner = inner;
    self->changed = g_signal_connect(inner, "changed", G_CALLBACK(monitor_changed), self);
    return G_FILE_MONITOR(self);
}
static GFileMonitor *monitor_dir(GFile *f, GFileMonitorFlags flags, GCancellable *c, GError **e) {
    return wrap_monitor(g_file_monitor_directory(unwrap(f), flags, c, e));
}
static GFileMonitor *monitor_file(GFile *f, GFileMonitorFlags flags, GCancellable *c, GError **e) {
    return wrap_monitor(g_file_monitor_file(unwrap(f), flags, c, e));
}
/* File operations stay on the underlying GFile/FUSE path. Its error and
 * cancellation semantics remain authoritative; this layer owns no journal. */
static GFileOutputStream *append_file(GFile *f, GFileCreateFlags flags, GCancellable *c, GError **e) {
    return g_file_append_to(unwrap(f), flags, c, e);
}
static GFileOutputStream *create_file(GFile *f, GFileCreateFlags flags, GCancellable *c, GError **e) {
    return g_file_create(unwrap(f), flags, c, e);
}
static GFileOutputStream *replace_file(GFile *f, const gchar *etag, gboolean backup,
                                     GFileCreateFlags flags, GCancellable *c, GError **e) {
    return g_file_replace(unwrap(f), etag, backup, flags, c, e);
}
static GFileIOStream *open_rw(GFile *f, GCancellable *c, GError **e) {
    return g_file_open_readwrite(unwrap(f), c, e);
}
static GFileIOStream *create_rw(GFile *f, GFileCreateFlags flags, GCancellable *c, GError **e) {
    return g_file_create_readwrite(unwrap(f), flags, c, e);
}
static GFileIOStream *replace_rw(GFile *f, const gchar *etag, gboolean backup,
                               GFileCreateFlags flags, GCancellable *c, GError **e) {
    return g_file_replace_readwrite(unwrap(f), etag, backup, flags, c, e);
}
static gboolean delete_file(GFile *f, GCancellable *c, GError **e) { return g_file_delete(unwrap(f), c, e); }
static gboolean trash_file(GFile *f, GCancellable *c, GError **e) { return g_file_trash(unwrap(f), c, e); }
static gboolean make_dir(GFile *f, GCancellable *c, GError **e) { return g_file_make_directory(unwrap(f), c, e); }
static gboolean make_link(GFile *f, const gchar *target, GCancellable *c, GError **e) {
    return g_file_make_symbolic_link(unwrap(f), target, c, e);
}
static gboolean copy_file(GFile *source, GFile *target, GFileCopyFlags flags, GCancellable *c,
                          GFileProgressCallback cb, gpointer data, GError **e) {
    return g_file_copy(unwrap(source), unwrap(target), flags, c, cb, data, e);
}
static gboolean move_file(GFile *source, GFile *target, GFileCopyFlags flags, GCancellable *c,
                          GFileProgressCallback cb, gpointer data, GError **e) {
    return g_file_move(unwrap(source), unwrap(target), flags, c, cb, data, e);
}
static GFile *rename_display(GFile *f, const gchar *name, GCancellable *c, GError **e) {
    return wrap(g_file_set_display_name(unwrap(f), name, c, e));
}
static GFileAttributeInfoList *settable(GFile *f, GCancellable *c, GError **e) {
    return g_file_query_settable_attributes(unwrap(f), c, e);
}
static GFileAttributeInfoList *writable(GFile *f, GCancellable *c, GError **e) {
    return g_file_query_writable_namespaces(unwrap(f), c, e);
}
static gboolean set_attr(GFile *f, const gchar *name, GFileAttributeType type, gpointer value,
                         GFileQueryInfoFlags flags, GCancellable *c, GError **e) {
    return g_file_set_attribute(unwrap(f), name, type, value, flags, c, e);
}
static gboolean set_attrs(GFile *f, GFileInfo *info, GFileQueryInfoFlags flags, GCancellable *c, GError **e) {
    return g_file_set_attributes_from_info(unwrap(f), info, flags, c, e);
}
static GMount *enclosing(GFile *f, GCancellable *c, GError **e) { return g_file_find_enclosing_mount(unwrap(f), c, e); }

static void file_iface_init(GFileIface *i) {
    i->dup=dup_file; i->hash=hash_file; i->equal=equal_file; i->is_native=native;
    i->has_uri_scheme=has_scheme; i->get_uri_scheme=get_uri_scheme;
    i->get_basename=get_basename; i->get_path=get_path; i->get_uri=get_uri; i->get_parse_name=get_parse_name;
    i->get_parent=get_parent; i->prefix_matches=prefix; i->get_relative_path=relative;
    i->resolve_relative_path=resolve; i->get_child_for_display_name=display_child;
    i->query_info=query_info; i->query_filesystem_info=query_fs; i->enumerate_children=enumerate;
    i->read_fn=read_file; i->monitor_dir=monitor_dir; i->monitor_file=monitor_file;
    i->append_to=append_file; i->create=create_file; i->replace=replace_file;
    i->open_readwrite=open_rw; i->create_readwrite=create_rw; i->replace_readwrite=replace_rw;
    i->delete_file=delete_file; i->trash=trash_file; i->make_directory=make_dir; i->make_symbolic_link=make_link;
    i->copy=copy_file; i->move=move_file; i->set_display_name=rename_display;
    i->query_settable_attributes=settable; i->query_writable_namespaces=writable;
    i->set_attribute=set_attr; i->set_attributes_from_info=set_attrs; i->find_enclosing_mount=enclosing;
    i->supports_thread_contexts=TRUE;
}


/* Derive from the installed backend instead of composing a local-only GVfs.
 * Only public path/URI constructors are overridden; inherited GVfs hooks keep
 * system metadata, trash and remote schemes on their original implementation.
 * Parent storage is opaque: GTypeQuery supplies sizes, no private layout copied. */
static GFile *path_file(GVfs *v, const gchar *p) { return wrap(base_vfs_class->get_file_for_path(v,p)); }
static GFile *uri_file(GVfs *v, const gchar *p) { return wrap(base_vfs_class->get_file_for_uri(v,p)); }
static GFile *parse_file(GVfs *v, const gchar *p) { return wrap(base_vfs_class->parse_name(v,p)); }
static void derived_class_init(gpointer klass, gpointer data) {
    (void)data;
    base_vfs_class = g_type_class_peek_parent(klass);
    GVfsClass *v = G_VFS_CLASS(klass);
    v->get_file_for_path=path_file; v->get_file_for_uri=uri_file; v->parse_name=parse_file;
}
G_MODULE_EXPORT void g_io_module_load(GIOModule *module) {
    /* Like GVfs itself, keep this module resident: GFile/monitor types and
     * their callbacks can outlive the Vfs extension's class reference. */
    g_type_module_use(G_TYPE_MODULE(module));
    GIOExtensionPoint *ep = g_io_extension_point_lookup(G_VFS_EXTENSION_POINT_NAME);
    if (!ep) return;
    /* Enumerate only already discovered modules. Rescanning their directory
     * would create duplicate GIOModule owners for the system's dynamic types.
     * Build/install must generate giomodule.cache before enabling this module. */
    GIOExtension *backend = g_io_extension_point_get_extension_by_name(ep,"gvfs");
    if (!backend) backend = g_io_extension_point_get_extension_by_name(ep,"local");
    if (!backend) return;
    GType parent = g_io_extension_get_type(backend);
    if (G_TYPE_IS_FINAL(parent)) return;
    gpointer parent_class = g_type_class_ref(parent);
    GTypeQuery query; g_type_query(parent,&query);
    if (!query.type || query.class_size > G_MAXUINT16 || query.instance_size > G_MAXUINT16) {
        g_type_class_unref(parent_class); return;
    }
    GTypeInfo info = {0};
    info.class_size=(guint16)query.class_size; info.instance_size=(guint16)query.instance_size;
    info.class_init=derived_class_init;
    GType type=g_type_module_register_type(G_TYPE_MODULE(module),parent,"ProbeVfs",&info,0);
    g_type_class_unref(parent_class);
    /* Select automatically when the module is discovered. Explicit user
     * GIO_USE_VFS choices still take precedence. Only verified live mounts
     * receive wrapped GFiles; ordinary paths keep the installed backend.
     * Test-scope builds must never automatically affect desktop processes. */
#ifdef DRIVE_NATIVE_GIO_TEST_SCOPES
    gint priority=-100;
#else
    gint parent_priority=g_io_extension_get_priority(backend);
    gint priority=parent_priority<G_MAXINT ? parent_priority+1 : G_MAXINT;
#endif
    g_io_extension_point_implement(G_VFS_EXTENSION_POINT_NAME,type,"drive-native-probe",priority);
}
G_MODULE_EXPORT void g_io_module_unload(GIOModule *module) { (void)module; }
G_MODULE_EXPORT gchar **g_io_module_query(void) {
    gchar **v=g_new0(gchar *,2);v[0]=g_strdup(G_VFS_EXTENSION_POINT_NAME);return v;
}
