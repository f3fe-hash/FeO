#include <net/server.h>

NetworkServer_t* listen_clients(const char* ip, const unsigned short port)
{
    NetworkServer_t* server = calloc(1, sizeof(NetworkServer_t));
    if (!server)
    {
        __global_err = ERR_FAILED_MALLOC;
        return NULL;
    }

    server->running = 1;
    net_init();

    server->ctx = SSL_CTX_new(TLS_server_method());
    if (!server->ctx)
    {
        __global_err = ERR_FAILED_CREATE_SSL_CTX;
        free(server);
        return NULL;
    }

    if (!SSL_CTX_use_certificate_file(server->ctx, CERT_PEM, SSL_FILETYPE_PEM))
    {
        __global_err = ERR_NO_CERT_FILE;
        ERR_print_errors_fp(stderr);
        printf("Failed to load certificate: %s\n", CERT_PEM);
        SSL_CTX_free(server->ctx);
        free(server);
        return NULL;
    }

    if (!SSL_CTX_use_PrivateKey_file(server->ctx, KEY_PEM, SSL_FILETYPE_PEM))
    {
        __global_err = ERR_NO_PRIV_FILE;
        ERR_print_errors_fp(stderr);
        printf("Failed to load private key: %s\n", KEY_PEM);
        SSL_CTX_free(server->ctx);
        free(server);
        return NULL;
    }

    server->serv_fd = socket(AF_INET, SOCK_STREAM, 0);
    if (server->serv_fd < 0)
    {
        __global_err = ERR_NO_SOCKET;
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
        if (inet_pton(AF_INET, ip, &server->addr.sin_addr) <= 0)
        {
            close(server->serv_fd);
            SSL_CTX_free(server->ctx);
            free(server);
            return NULL;
        }
    }

    int opt = 1;
    setsockopt(server->serv_fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));

    if (bind(server->serv_fd, (struct sockaddr *)&server->addr, sizeof(server->addr)) < 0)
    {
        __global_err = ERR_FAILED_BIND;
        close(server->serv_fd);
        SSL_CTX_free(server->ctx);
        free(server);
        return NULL;
    }

    if (listen(server->serv_fd, 5) < 0)
    {
        __global_err = ERR_FAILED_LISTEN;
        close(server->serv_fd);
        SSL_CTX_free(server->ctx);
        free(server);
        return NULL;
    }

    return server;
}

void stop_server(NetworkServer_t* server)
{
    server->running = 0;
    pthread_join(server->thread, NULL);

    SSL_CTX_free(server->ctx);
}

void set_server_response(NetworkServer_t* server, int(* func)(NetworkClientConnection_t *))
{
    server->response = func;
}

void* server_thread_func(void* arg)
{
    NetworkServer_t* server = (NetworkServer_t *)arg;
    int err_count = 0;

    while (server->running)
    {
        int client_fd = accept(server->serv_fd, NULL, NULL);
        if (client_fd < 0)
        {
            __global_err = ERR_FAILED_ACCEPT;
            err_count++;
            continue;
        }

        SSL* ssl = SSL_new(server->ctx);
        if (!ssl)
        {
            __global_err = ERR_FAILED_CREATE_SSL;
            err_count++;
            close(client_fd);
            continue;
        }

        SSL_set_fd(ssl, client_fd);

        if (SSL_accept(ssl) <= 0)
        {
            __global_err = ERR_FAILED_ACCEPT;
            err_count++;
            goto cleanup;
        }

        NetworkClientConnection_t* conn =
            (NetworkClientConnection_t *)calloc(1, sizeof(NetworkClientConnection_t));

        if (!conn)
        {
            __global_err = ERR_FAILED_MALLOC;
            err_count++;
            goto cleanup;
        }

        conn->sockfd = client_fd;
        conn->ssl = ssl;

        int ret = server->response(conn);
        (void)ret;

        free(conn);

cleanup:
        SSL_shutdown(ssl);
        SSL_free(ssl);
        close(client_fd);

        // Errors might just be accidents
        if (err_count >= 5)
            break;
    }

    return NULL;
}

void run_server(NetworkServer_t* server)
{
    pthread_t thread;

    int ret = pthread_create(
        &thread,
        NULL,
        server_thread_func,
        server
    );

    if (ret != 0)
    {
        __global_err = ERR_FAILED_CREATE_THREAD;
        return;
    }
    server->thread = thread;
}

void write_server(NetworkClientConnection_t* conn, const char* data, size_t len)
{
    char header[64];
    int header_len = snprintf(header, sizeof(header), "Content-Length: %zu\r\n\r\n", len);
    if (header_len < 0)
    {
        __global_err = ERR_INVALID_WRITE;
        return;
    }

    // Send header (handle partial writes)
    int sent = 0;
    while (sent < header_len)
    {
        int n = SSL_write(conn->ssl, header + sent, header_len - sent);
        if (n <= 0)
        {
            __global_err = ERR_INVALID_WRITE;
            return;
        }
        sent += n;
    }

    // Send body in chunks to avoid large stack allocations
    size_t total_sent = 0;
    while (total_sent < len)
    {
        size_t to_write = len - total_sent;
        if (to_write > 16384)
            to_write = 16384;

        int n = SSL_write(conn->ssl, data + total_sent, (int)to_write);
        if (n <= 0)
        {
            __global_err = ERR_INVALID_WRITE;
            return;
        }
        total_sent += (size_t)n;
    }
}

char* read_server(NetworkClientConnection_t* conn)
{
    char header[10000];
    int total = 0;
    int header_end = -1; // index of byte just after end of header

    while (header_end < 0)
    {
        int bytes = SSL_read(conn->ssl, header + total, (int)(sizeof(header) - total));
        if (bytes <= 0)
        {
            __global_err = ERR_INVALID_READ;
            return NULL;
        }
        total += bytes;

        // search for "\r\n\r\n"
        for (int i = 0; i <= total - 4; ++i)
        {
            if (header[i] == '\r' && header[i + 1] == '\n' && header[i + 2] == '\r' && header[i + 3] == '\n')
            {
                header_end = i + 4;
                break;
            }
        }

        if (total == (int)sizeof(header) && header_end < 0)
        {
            __global_err = ERR_INVALID_READ;
            return NULL;
        }
    }

    size_t len = 0;
    char* cl = strstr(header, "Content-Length:");
    if (cl)
    {
        cl += strlen("Content-Length:");
        while (*cl == ' ' || *cl == '\t') cl ++;
        sscanf(cl, "%zu", &len);
    }
    else
    {
        // Fallback: try to sscanf from start
        sscanf(header, "Content-Length: %zu", &len);
    }

    char* data = calloc(len + 1, 1);
    if (!data)
    {
        __global_err = ERR_FAILED_MALLOC;
        return NULL;
    }

    int header_remaining = total - header_end;
    if (header_remaining > 0)
    {
        int copy_len = header_remaining;
        if ((size_t)copy_len > len) copy_len = (int)len;
        memcpy(data, header + header_end, copy_len);
    }

    size_t received = (header_remaining > 0) ? (size_t)header_remaining : 0;
    while (received < len)
    {
        int toread = (int)(len - received);
        if (toread > 16384) toread = 16384;
        int bytes = SSL_read(conn->ssl, data + received, toread);
        if (bytes <= 0)
        {
            __global_err = ERR_INVALID_READ;
            free(data);
            return NULL;
        }
        received += (size_t)bytes;
    }

    data[len] = '\0';
    return data;
}

void free_buffer(char* buffer)
{
    free(buffer);
}
