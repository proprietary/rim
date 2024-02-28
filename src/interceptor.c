#include <dlfcn.h>
#include <stdio.h>
#include <unistd.h>

static int (*original_unlink)(const char *) = NULL;
static int (*original_unlinkat)(int fd, const char *path, int flag) = NULL;

int unlink(const char *pathname) {
  printf("Intercepting unlink(2): %s\n", pathname);
  if (original_unlink == NULL) {
    void *libc_handle = dlopen("libc.dylib", RTLD_LAZY);
    if (!libc_handle) {
      fprintf(stderr, "could not find libc.dylib\n");
      return -1;
    }
    original_unlink = (int (*)(const char *))dlsym(libc_handle, "unlink");
    if (!original_unlink) {
      fprintf(stderr, "could not find the symbol for unlink(2)\n");
      dlclose(libc_handle);
      return -1;
    }
    printf("Executing custom logic...\n");
    dlclose(libc_handle);
  }
  return original_unlink(pathname);
}

int unlinkat(int fd, const char *path, int flag) {
  printf("intercepting unlinkat(2): (fd=%d, path=%s, flag=%d)\n", fd, path,
         flag);
  printf("Executing custom logic...");
  if (original_unlinkat == NULL) {
    void *libc_handle = dlopen("libc.dylib", RTLD_LAZY);
    if (!libc_handle) {
      return -1;
    }
    original_unlinkat =
        (int (*)(int, const char *, int))dlsym(libc_handle, "unlinkat");
    if (!original_unlinkat) {
      dlclose(libc_handle);
      return -1;
    }
    dlclose(libc_handle);
  }
  int result = unlinkat(fd, path, flag);
  return result;
}

__attribute__((constructor)) static void init_interceptor() {
  printf("hello!\n");
  original_unlinkat = dlsym(RTLD_NEXT, "unlinkat");
  original_unlink = dlsym(RTLD_NEXT, "unlink");
}
