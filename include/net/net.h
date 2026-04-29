#pragma once
#ifdef __cplusplus
extern "C"
{
#endif

// Stdlib
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <arpa/inet.h>

// OpenSSL
#include <openssl/ssl.h>
#include <openssl/err.h>

// Processes
#include <unistd.h>
#include <signal.h>

void net_init()
{
    static int initialize_net = 0;
    if (initialize_net == 0)
    {
#if OPENSSL_API_COMPAT <= 0x10100000L
        SSL_library_init();
#endif
        OpenSSL_add_all_algorithms();
        SSL_load_error_strings();
        initialize_net = 1;
    }
}

#ifdef __cplusplus
}
#endif