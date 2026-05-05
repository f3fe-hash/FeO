#include <net/server.h>

NetworkServer_t* listen_clients(const char* ip, const unsigned short port)
{
    NetworkServer_t* server = calloc(1, sizeof(NetworkServer_t));
    if (!server)
    {
        __global_err = ERR_FAILED_MALLOC;
        return NULL;
    }

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
    if (kill(server->pid, SIGTERM))
    {
        __global_err = ERR_FAILED_TERMINATE;
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
    int err_count;
    server->pid = fork();
    if (server->pid == 0)
    {
        while (1)
        {
            int client_fd = accept(server->serv_fd, NULL, NULL);
            if (client_fd < 0)
            {
                __global_err = ERR_FAILED_ACCEPT;
                err_count ++;
                continue;
            }

            SSL* ssl = SSL_new(server->ctx);
            if (!ssl)
            {
                __global_err = ERR_FAILED_CREATE_SSL;
                err_count ++;
                close(client_fd);
                continue;
            }
            SSL_set_fd(ssl, client_fd);

            if (SSL_accept(ssl) <= 0)
            {
                __global_err = ERR_FAILED_ACCEPT;
                err_count ++;
                goto cleanup;
            }

            NetworkClientConnection_t* conn = (NetworkClientConnection_t *)calloc(1, sizeof(NetworkClientConnection_t));
            if (! conn)
            {
                __global_err = ERR_FAILED_MALLOC;
                err_count ++;
                goto cleanup;
            }

            conn->sockfd = client_fd;
            conn->ssl = ssl;

            int ret = server->response(conn);
            (void) ret;
            free(conn);

cleanup:
            SSL_shutdown(ssl);
            SSL_free(ssl);
            close(client_fd);

            // Errors might just be accidents
            if (err_count < 5)
                continue;
            else // More than 5 errors. Stop the server.
                break;
        }
    }
}

void write_server(NetworkClientConnection_t* conn, const char* data, size_t len)
{
    // Create header
    // Format string is 22 chars. Use the logarithm of
    // the length to get the number of characters the number takes up.
    int header_len = (int)log10(len) + 22;
    char header[header_len];
    snprintf(header, header_len, "Content-Length: %ul\r\n", len);

    // Combine header with data
    size_t final_len = header_len + len;
    char final_data[final_len + 1];
    memcpy(final_data, header, header_len);
    memcpy(final_data[header_len], data, len);
    final_data[final_len] = '\0';

    // Write it all
    int remaining = final_len; // # of remaining bytes
    int n = 1; // Bytes written this write call
    while ((remaining > 0) && (n > 0))
    {
        n = SSL_write(conn->ssl, final_data, len);
        if (n < 0)
        {
            __global_err = ERR_INVALID_WRITE;
            return;
        }
        
        remaining -= n;
    }
}

char* read_server(NetworkClientConnection_t* conn)
{
    char header[10000];
    int pos = 0;
    int reached_end_header = 0;
    int total_bytes = 0;
    while (! reached_end_header)
    {
        int bytes = SSL_read(conn->ssl, &header[total_bytes], 30);
        if (bytes < 0)
        {
            __global_err = ERR_INVALID_READ;
            return NULL;
        }

        total_bytes += bytes;
        for (int i = 0; i < bytes; i ++)
        {
            if (header[pos] == '\n')
                reached_end_header = 2; // Move 2 bytes to data
            else if (header[pos] == '\r')
                reached_end_header = 1; // Move 1 byte to data
            pos ++;
        }
    }

    // Compute and allocate size
    size_t len;
    sscanf(header, "Content-Length: %ul\r\n", len);
    char* data = calloc(len, sizeof(char));
    if (!data)
    {
        __global_err = ERR_FAILED_MALLOC;
        return NULL;
    }

    // Copy the excess bytes to data. Accidentally read them into header.
    int header_written = total_bytes - pos;
    memcpy(data, header[pos + reached_end_header], header_written);

    // Read the rest of the bytes
    int remaining = len - header_written;
    int byte_idx = header_written;
    while (remaining > 0)
    {
        int bytes = SSL_read(conn->ssl, &data[byte_idx], 10000);
        if (bytes < 0)
        {
            __global_err = ERR_INVALID_READ;
            return NULL;
        }

        remaining -= bytes;
    }
}

void free_buffer(char* buffer)
{
    free(buffer);
}
