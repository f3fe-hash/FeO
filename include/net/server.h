#pragma once
#ifdef __cplusplus
extern "C"
{
#endif

#include <net/net.h>

typedef struct net_cli_conn_t
{
    int sockfd;
    SSL* ssl;
} NetworkClientConnection_t;

typedef struct net_server_t
{
    SSL_CTX* ctx;
    struct sockaddr_in addr;
    int serv_fd;
    pid_t pid;

    int (* response)(NetworkClientConnection_t *);
} NetworkServer_t;

void set_server_response(NetworkServer_t* server, int(* func)(NetworkClientConnection_t *));

NetworkServer_t* listen_clients(const char* ip, const unsigned short port);
void stop_server(NetworkServer_t* server);
void run_server(NetworkServer_t* server);

int write_server(NetworkClientConnection_t* conn, const char* data, size_t len);

#ifdef __cplusplus
}
#endif