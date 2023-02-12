#include <stdio.h>
#include <string.h>
#include <dlfcn.h>

int main(int argc, char *argv[])
{
    if (argc > 2)
    {
        const char *library_path = argv[1];
        const char *symbol_name = argv[2];

        if (strcmp("__dummy_mirror_test", library_path) == 0)
        {
            printf("%s %s", library_path, symbol_name);
            return 0;
        }

        void *handle = dlopen(library_path, RTLD_LAZY);
        if (!handle)
        {
            fputs(dlerror(), stderr);
            return 10;
        }

        void (*fn)() = dlsym(handle, symbol_name);
        char *error = NULL;

        if ((error = dlerror()) != NULL)
        {
            fputs(error, stderr);
            return 11;
        }
        (*fn)();
        dlclose(handle);
        return 0;
    }

    return 9;
}