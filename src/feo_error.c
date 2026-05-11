#include <feo_error.h>

__thread int __global_err = ERR_OK;

void _set_error(int err)
{
    __global_err = err;
}

int _get_error()
{
    return __global_err;
}
