#pragma once
#ifdef __cplusplus
extern "C"
{
#endif

#include <net/net.h>

typedef struct net_client_t
{
    int sockfd;
    struct sockaddr_in addr;

    SSL* ssl;
    SSL_CTX* ctx;
} NetworkClient_t;

NetworkClient_t* connect_server(const char* ip, const short port);
void disconnect_server(NetworkClient_t* cli);

int write_server(NetworkClient_t* cli, const char* request);
int writen_server(NetworkClient_t* cli, const char* request, size_t len);

int read_server(NetworkClient_t* cli, char** buff, size_t n);

#ifdef __cplusplus
}
#endif