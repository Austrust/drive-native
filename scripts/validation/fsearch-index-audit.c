/* Read-only verifier linked to the unchanged upstream 0.3.1 library.
 * This does not scan the mount or build an index. FSearch itself does that. */
#include "fsearch_database_file.h"
#include "fsearch_database_chunked_array.h"
#include "fsearch_database_entry.h"
#include <stdio.h>
#include <string.h>
#include <sys/types.h>
static int dump(FsearchDatabaseChunkedArray *entries, int directory) {
    uint32_t count=fsearch_database_chunked_array_get_num_entries(entries);
    for(uint32_t i=0;i<count;i++) {
        FsearchDatabaseEntry *e=fsearch_database_chunked_array_get_entry(entries,i);
        GString *p=db_entry_get_path_full(e);
        time_t mtime=0;off_t size=0;
        if(!p || strchr(p->str,'\t') || strchr(p->str,'\n') ||
           !db_entry_get_attribute(e,DATABASE_INDEX_PROPERTY_MODIFICATION_TIME,&mtime,sizeof(mtime)) ||
           (!directory && !db_entry_get_attribute(e,DATABASE_INDEX_PROPERTY_SIZE,&size,sizeof(size)))) return 5;
        printf("%d\t%lld\t%lld\t%s\n",directory,(long long)mtime,(long long)size,p->str);
        g_string_free(p,TRUE);
    }
    return 0;
}
int main(int argc,char **argv) {
    if(argc!=2)return 2;
    FsearchDatabaseIncludeManager *includes=NULL;
    FsearchDatabaseExcludeManager *excludes=NULL;
    FsearchDatabaseIndexPropertyFlags flags=0;
    FsearchDatabaseIndexStore *store=NULL;
    if(!fsearch_database_file_load_config(argv[1],&includes,&excludes,&flags))return 3;
    if(!fsearch_database_file_load(argv[1],NULL,&store,includes,excludes,NULL,NULL))return 4;
    int result=dump(fsearch_database_index_store_get_folders(store,DATABASE_INDEX_PROPERTY_NAME),1);
    if(!result)result=dump(fsearch_database_index_store_get_files(store,DATABASE_INDEX_PROPERTY_NAME),0);
    fsearch_database_index_store_unref(store);
    return result;
}
