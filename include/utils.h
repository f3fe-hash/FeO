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

// Combine N paths into a single path with '/' separators.
// Example usage: `char *p = join_paths(2, "a", "b");`
char* join_paths(int count, ...);

#ifdef __cplusplus
}
#endif