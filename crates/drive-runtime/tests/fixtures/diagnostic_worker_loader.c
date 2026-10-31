/* Test-only ELF entry point for the Python IPC peer. The production worker is
 * ELF too: its pinned image fd must remain CLOEXEC, which is incompatible with
 * passing a /proc/self/fd path as the script name to a shebang interpreter. */
#define _POSIX_C_SOURCE 200809L
#include <limits.h>
#include <stddef.h>
#include <string.h>
#include <unistd.h>

int main(int argc, char **argv) {
    char image[PATH_MAX + 1];
    const char script[] = "diagnostic_worker.py";
    if (argc != 2 || strcmp(argv[1], "--ipc-stdin") != 0)
        return 120;
    ssize_t size = readlink("/proc/self/exe", image, sizeof(image) - 1);
    if (size <= 0 || size >= (ssize_t)(sizeof(image) - 1))
        return 121;
    image[size] = '\0';
    char *name = strrchr(image, '/');
    if (!name || (size_t)(name + 1 - image) + sizeof(script) > sizeof(image))
        return 122;
    /* A replaced executable may end in " (deleted)". Its parent still names
     * this isolated peer directory; never reopen the replacement executable. */
    memcpy(name + 1, script, sizeof(script));
    execl("/usr/bin/python3", "python3", image, argv[1], (char *)NULL);
    return 123;
}
