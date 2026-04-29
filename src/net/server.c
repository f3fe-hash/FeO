#include <net/server.h>

NetworkServer_t* listen_clients(const char* ip, const short port)
{
    NetworkServer_t* server = calloc(1, sizeof(NetworkServer_t));
    if (!server) return NULL;

    net_init();

    server->ctx = SSL_CTX_new(TLS_server_method());
    if (!server->ctx)
    {
        free(server);
        return NULL;
    }

    if (!SSL_CTX_use_certificate_file(server->ctx, "cert.pem", SSL_FILETYPE_PEM))
    {
        SSL_CTX_free(server->ctx);
        free(server);
        return NULL;
    }

    if (!SSL_CTX_use_PrivateKey_file(server->ctx, "key.pem", SSL_FILETYPE_PEM))
    {
        SSL_CTX_free(server->ctx);
        free(server);
        return NULL;
    }

    server->serv_fd = socket(AF_INET, SOCK_STREAM, 0);
    if (server->serv_fd < 0)
    {
        SSL_CTX_free(server->ctx);
        free(server);
        return NULL;
    }

    server->addr.sin_family = AF_INET;
    server->addr.sin_port = htons(port);

    if (!ip)
    {
        server->addr.sin_addr.s_addr = INADDR_ANY;
    }
    else
    {
        if (inet_pton(AF_INET, ip, &server->addr.sin_addr) <= 0) {
            close(server->serv_fd);
            SSL_CTX_free(server->ctx);
            free(server);
            return NULL;
        }
    }

    if (bind(server->serv_fd, (struct sockaddr *)&server->addr, sizeof(server->addr)) < 0)
    {
        close(server->serv_fd);
        SSL_CTX_free(server->ctx);
        free(server);
        return NULL;
    }

    if (listen(server->serv_fd, 5) < 0)
    {
        close(server->serv_fd);
        SSL_CTX_free(server->ctx);
        free(server);
        return NULL;
    }

    return server;
}

void stop_server(NetworkServer_t* server)
{
    if (kill(server->pid, SIGTERM))
    {
        perror("Failed to terminate server process");
    }

    SSL_CTX_free(server->ctx);
}

void set_server_response(NetworkServer_t* server, int(* func)(NetworkClientConnection_t *))
{
    server->response = func;
}

void run_server(NetworkServer_t* server)
{
    server->pid = fork();
    if (server->pid == 0)
    {
        while (1)
        {
            int client_fd = accept(server->serv_fd, NULL, NULL);
            if (client_fd < 0)
            {
                perror("accept failed");
                continue;
            }

            SSL* ssl = SSL_new(server->ctx);
            if (!ssl)
            {
                close(client_fd);
                continue;
            }
            SSL_set_fd(ssl, client_fd);

            if (SSL_accept(ssl) <= 0)
            {
                SSL_free(ssl);
                close(client_fd);
                continue;
            }

            NetworkClientConnection_t* conn = (NetworkClientConnection_t *)calloc(1, sizeof(NetworkClientConnection_t));
            conn->sockfd = client_fd;
            conn->ssl = ssl;

            (void) server->response(conn);
            free(conn);

            SSL_shutdown(ssl);
            SSL_free(ssl);
            close(client_fd);
        }
    }
}
