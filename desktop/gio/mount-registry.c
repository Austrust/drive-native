#define _GNU_SOURCE
#include "mount-registry.h"
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <linux/stat.h>
#include <unistd.h>

static gboolean beneath(const gchar *path, const gchar *root) {
    if (!path || !root || root[0]!='/' || path[0]!='/') return FALSE;
    gsize n=strlen(root);
    return g_str_has_prefix(path,root) && (path[n]=='\0' || path[n]=='/' || n==1);
}
static int private_dir(int parent, const gchar *name) {
    int fd=parent<0 ? open(name,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC)
                    : openat(parent,name,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);
    if (fd<0) return -1;
    struct stat st;
    if (fstat(fd,&st) || !S_ISDIR(st.st_mode) || st.st_uid!=geteuid() || (st.st_mode&077)) {
        close(fd);return -1;
    }
    return fd;
}
static gboolean decimal(const gchar *text, guint64 *out) {
    if (!text || !*text) return FALSE;
    for (const gchar *p=text;*p;p++) if (!g_ascii_isdigit(*p)) return FALSE;
    errno=0;gchar *end=NULL;guint64 n=g_ascii_strtoull(text,&end,10);
    if (errno || !end || *end) return FALSE;
    *out=n;return TRUE;
}
static gboolean key_number(GKeyFile *key, const gchar *name, guint64 *out) {
    gchar *text=g_key_file_get_value(key,"Mount",name,NULL);
    gboolean ok=decimal(text,out);g_free(text);return ok;
}
static gchar *decode_hex(const gchar *text) {
    if (!text || !*text || strlen(text)%2) return NULL;
    gsize len=strlen(text)/2;gchar *out=g_malloc(len+1);
    for (gsize i=0;i<len;i++) {
        int a=g_ascii_xdigit_value(text[i*2]),b=g_ascii_xdigit_value(text[i*2+1]);
        if (a<0 || b<0 || !(a*16+b)) {g_free(out);return NULL;}
        out[i]=(gchar)(a*16+b);
    }
    out[len]='\0';return out;
}
static gchar *unescape_mount(const gchar *text) {
    GString *out=g_string_new(NULL);
    for (gsize i=0;text[i];i++) {
        if (text[i]=='\\') {
            unsigned n=0;
            for (int j=1;j<=3;j++) {
                if (!text[i+j] || text[i+j]<'0' || text[i+j]>'7') {g_string_free(out,TRUE);return NULL;}
                n=n*8+(unsigned)(text[i+j]-'0');
            }
            if (!n || n>255) {g_string_free(out,TRUE);return NULL;}
            g_string_append_c(out,(gchar)n);i+=3;
        } else g_string_append_c(out,text[i]);
    }
    return g_string_free(out,FALSE);
}
static gboolean process_matches(guint64 pid,guint64 start) {
    if (!pid || pid>G_MAXINT) return FALSE;
    gchar *proc=g_strdup_printf("/proc/%"G_GUINT64_FORMAT,pid);
    struct stat st;
    if (stat(proc,&st) || st.st_uid!=geteuid()) {g_free(proc);return FALSE;}
    gchar *file=g_build_filename(proc,"stat",NULL);g_free(proc);
    gchar *data=NULL;gsize len=0;
    gboolean ok=g_file_get_contents(file,&data,&len,NULL);g_free(file);
    if (!ok || len>8192) {g_free(data);return FALSE;}
    gchar *end=strrchr(data,')');gboolean matched=FALSE;
    if (end) {
        gchar **parts=g_strsplit_set(end+1," \t\n",-1);unsigned field=0;
        for (gchar **p=parts;*p;p++) if (**p) {
            if (field++==19) {guint64 actual;matched=decimal(*p,&actual) && actual==start;break;}
        }
        g_strfreev(parts);
    }
    g_free(data);return matched;
}
static gboolean kernel_matches(const gchar *mountinfo,const gchar *root,guint64 id,const gchar *path) {
    gboolean found=FALSE,overlaid=FALSE;
    gchar **lines=g_strsplit(mountinfo,"\n",-1);
    for (gchar **line=lines;*line;line++) {
        gchar *split=strstr(*line," - ");if (!split) continue;
        *split='\0';gchar **head=g_strsplit(*line," ",-1),**tail=g_strsplit(split+3," ",-1);
        if (g_strv_length(head)>=6 && g_strv_length(tail)>=2) {
            gchar *at=unescape_mount(head[4]);guint64 actual;
            if (at && decimal(head[0],&actual)) {
                if (!strcmp(at,root) && actual==id &&
                    (!strcmp(tail[0],"fuse") || !strcmp(tail[0],"fuse.drive-native")) && !strcmp(tail[1],"drive-native")) found=TRUE;
                /* Nested or replacement mounts must never inherit this scope. */
                if (actual!=id && beneath(at,root) && beneath(path,at)) overlaid=TRUE;
            }
            g_free(at);
        }
        g_strfreev(head);g_strfreev(tail);
    }
    g_strfreev(lines);return found && !overlaid;
}
/* Legacy mount IDs can be reused immediately after unmount. A unique statx
 * mount ID is never reused during this boot (Linux 6.8+). */
static gboolean unique_matches(const gchar *path,const gchar *root,guint64 expected) {
    gchar *candidate=g_strdup(path);gboolean matched=FALSE;
    for (unsigned depth=0;depth<256 && beneath(candidate,root);depth++) {
        struct statx value;
        if (!statx(AT_FDCWD,candidate,AT_NO_AUTOMOUNT|AT_SYMLINK_NOFOLLOW|AT_STATX_DONT_SYNC,
                   STATX_MNT_ID_UNIQUE,&value)) {
            matched=(value.stx_mask&STATX_MNT_ID_UNIQUE) && value.stx_mnt_id==expected;break;
        }
        if (errno!=ENOENT || !strcmp(candidate,root)) break;
        gchar *parent=g_path_get_dirname(candidate);g_free(candidate);candidate=parent;
    }
    g_free(candidate);return matched;
}
/* Invalidate the mount snapshot on kernel notifications, never a timer.
 * /proc/self/mounts is pollable even when an ID is removed and reused between
 * queries. Keep no descriptor on the FUSE mount itself (which would pin it).
 * The current FUSE namespace rejects symlinks; nested mounts are checked below. */
static GMutex mount_cache_lock;
static int mount_watch=-1;
static pid_t cache_pid;
static ino_t cache_namespace;
static dev_t cache_namespace_device;
static gchar *mount_snapshot;
static GHashTable *validated_roots;
static gboolean live_mount_matches(const gchar *root,guint64 id,guint64 unique,const gchar *path) {
    gboolean matched=FALSE;
    g_mutex_lock(&mount_cache_lock);
    struct stat ns;
    if (stat("/proc/self/ns/mnt",&ns)) goto done;
    gboolean changed=mount_watch<0 || cache_pid!=getpid() || cache_namespace!=ns.st_ino || cache_namespace_device!=ns.st_dev;
    if (!changed) {
        struct pollfd watch={.fd=mount_watch,.events=POLLPRI};
        int result;
        do { result=poll(&watch,1,0); } while (result<0 && errno==EINTR);
        if (result<0) goto done;
        changed=(watch.revents&(POLLPRI|POLLERR|POLLHUP|POLLNVAL))!=0;
    }
    if (changed) {
        if (mount_watch>=0) close(mount_watch);
        mount_watch=open("/proc/self/mounts",O_RDONLY|O_CLOEXEC);
        g_clear_pointer(&mount_snapshot,g_free);
        if (validated_roots) g_hash_table_remove_all(validated_roots);
        cache_pid=getpid();cache_namespace=ns.st_ino;cache_namespace_device=ns.st_dev;
        if (mount_watch<0) goto done;
        gsize length=0;
        if (!g_file_get_contents("/proc/self/mountinfo",&mount_snapshot,&length,NULL) || length>4*1024*1024) {
            g_clear_pointer(&mount_snapshot,g_free);close(mount_watch);mount_watch=-1;goto done;
        }
    }
    if (!mount_snapshot) goto done;
    if (!validated_roots) validated_roots=g_hash_table_new_full(g_str_hash,g_str_equal,g_free,NULL);
    gchar *key=g_strdup_printf("%"G_GUINT64_FORMAT":%s",unique,root);
    gboolean valid=g_hash_table_contains(validated_roots,key);
    if (!valid && unique_matches(root,root,unique)) {
        g_hash_table_add(validated_roots,key);key=NULL;valid=TRUE;
    }
    g_free(key);
    if (valid) matched=kernel_matches(mount_snapshot,root,id,path);
done:
    g_mutex_unlock(&mount_cache_lock);return matched;
}
static gboolean record_matches(int directory,const gchar *name,const gchar *path) {
    int fd=openat(directory,name,O_RDONLY|O_NOFOLLOW|O_CLOEXEC|O_NONBLOCK);
    if (fd<0) return FALSE;
    struct stat st;
    if (fstat(fd,&st) || !S_ISREG(st.st_mode) || st.st_uid!=geteuid() || (st.st_mode&077)
        || st.st_nlink!=1 || st.st_size<1 || st.st_size>8192) {close(fd);return FALSE;}
    gchar data[8193];gsize size=0;
    while (size<sizeof data) {
        ssize_t got=read(fd,data+size,sizeof data-size);
        if (got<0 && errno==EINTR) continue;
        if (got<0) {close(fd);return FALSE;}
        if (!got) break;
        size+=(gsize)got;
    }
    close(fd);if (size>8192) return FALSE;
    GKeyFile *key=g_key_file_new();gboolean matched=FALSE;
    if (g_key_file_load_from_data(key,data,size,G_KEY_FILE_NONE,NULL)) {
        guint64 schema,uid,pid,start,id,unique;
        if (key_number(key,"schema",&schema) && schema==1 && key_number(key,"uid",&uid) && uid==geteuid() &&
            key_number(key,"pid",&pid) && key_number(key,"pid_start",&start) && key_number(key,"mount_id",&id) && key_number(key,"mount_unique_id",&unique)) {
            gchar *hex=g_key_file_get_value(key,"Mount","path_hex",NULL);
            gchar *root=decode_hex(hex);g_free(hex);
            if (root && beneath(path,root) && process_matches(pid,start))
                matched=live_mount_matches(root,id,unique,path);
            g_free(root);
        }
    }
    g_key_file_unref(key);return matched;
}
gboolean dn_scope_contains_path(const gchar *path) {
    if (!path || path[0]!='/') return FALSE;
#ifdef DRIVE_NATIVE_GIO_TEST_SCOPES
    /* Compile-time opt-in for disposable local fixtures, never a shipping build. */
    const gchar *test_root=g_getenv("DRIVE_NATIVE_METADATA_ROOT");
    if (test_root && beneath(path,test_root)) return TRUE;
#endif
    const gchar *runtime=g_getenv("XDG_RUNTIME_DIR");
    if (!runtime || runtime[0]!='/') return FALSE;
    gchar *canonical=g_canonicalize_filename(runtime,NULL),*resolved=realpath(runtime,NULL);
    gboolean same=resolved && !strcmp(canonical,resolved);g_free(canonical);free(resolved);
    if (!same) return FALSE;
    int root=private_dir(-1,runtime);if (root<0) return FALSE;
    int app=private_dir(root,"drive-native");close(root);if (app<0) return FALSE;
    int mounts=private_dir(app,"mounts");close(app);if (mounts<0) return FALSE;
    DIR *directory=fdopendir(mounts);if (!directory) {close(mounts);return FALSE;}
    gboolean result=FALSE;
    struct dirent *entry;unsigned count=0;
    while ((entry=readdir(directory))!=NULL && count++<512) {
        if (strlen(entry->d_name)!=42 || strcmp(entry->d_name+36,".mount")) continue;
        gchar *uuid=g_strndup(entry->d_name,36);gboolean valid=g_uuid_string_is_valid(uuid);g_free(uuid);
        if (valid && record_matches(mounts,entry->d_name,path)) {result=TRUE;break;}
    }
    closedir(directory);return result;
}
