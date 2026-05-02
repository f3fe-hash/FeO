#pragma once
#ifdef __cplusplus
extern "C"
{
#endif

#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>

// Errors
#include <feo_error.h>

// Combine 2 paths: path1 and path2 like this: <path1>/<path2>
char* join_paths(int count, ...)
{
    va_list args;
    
    // 1. Calculate total length for allocation
    va_start(args, count);
    size_t total_len = 0;
    for (int i = 0; i < count; i++)
    {
        const char* s = va_arg(args, const char*);
        if (s) total_len += strlen(s);
    }
    va_end(args);

    // Allocation size: total string length + max possible separators (count-1) + null terminator
    char* result = (char *)calloc(total_len + count, sizeof(char));
    if (!result) return NULL;
    result[0] = '\0';

    // 2. Concatenate paths
    va_start(args, count);
    for (int i = 0; i < count; i++)
    {
        const char* s = va_arg(args, const char*);
        if (!s) continue;

        // Append path segment
        strcat(result, s);

        // Add separator if not the last item and segment doesn't already have one
        if (i < count - 1)
        {
            size_t curr_len = strlen(result);
            if (curr_len > 0 && result[curr_len - 1] != '/')
                strcat(result, "/");
        }
    }
    va_end(args);

    return result;
}

#ifdef __cplusplus
}
#endif