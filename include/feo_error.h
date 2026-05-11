#pragma once
#ifdef __cplusplus
extern "C"
{
#endif

#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>

// System errors
#define ERR_OK 0x00 // No errors
#define ERR_FAILED_MALLOC 0x01 // Failed to allocate memory. Works for any allocation function, not just malloc.
#define ERR_FAILED_CREATE_THREAD 0x02 // Failed to create a thread.

// Process errors
#define ERR_EXECLP 0x10 // Failed using execlp
#define ERR_MAX_PROC_REACHED 0x11 // Maximum processs limit reached
#define ERR_FAILED_TERMINATE 0x12 // Failed to terminate process

// Networking errors
#define ERR_INVALID_WRITE 0x20 // Failed using SSL_write
#define ERR_INVALID_READ 0x21 // Failed using SSL_read
#define ERR_FAILED_CREATE_SSL_CTX 0x22 // Failed to create an ssl context
#define ERR_FAILED_CREATE_SSL 0x23 // Failed to create an ssl
#define ERR_NO_CERT_FILE 0x24 // Failed to use certificate file
#define ERR_NO_PRIV_FILE 0x25 // Failed to use private key file
#define ERR_NO_SOCKET 0x26 // Failed to create a socket
#define ERR_FAILED_BIND 0x27 // Failed to bind socket
#define ERR_FAILED_LISTEN 0x28 // Failed to listen on socket
#define ERR_FAILED_ACCEPT 0x29 // Failed to accept client

extern __thread int __global_err;

void _set_error(int err);
int _get_error();

#ifdef __cplusplus
}
#endif