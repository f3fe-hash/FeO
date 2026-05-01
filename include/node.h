#pragma once
#ifdef __cplusplus
extern "C"
{
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Processes
#include <unistd.h>
#include <errno.h>
#include <sys/signal.h>
#include <sys/stat.h>

// Custom Utilities
#include <utils.h>

#define PROC_CLEANUP_DURATION_US 1000 // Wait 1 ms for process cleanup before forcefully killing it.

#define NODE_DIR "/etc/nodes"

typedef struct feo_node_t
{
    pid_t pid;
    char* name;
    int name_len;
} Node_t;

void init_nodes()
{
    mkdir(NODE_DIR, 0777);
}

Node_t* create_node(const char* name)
{
    Node_t* node = (Node_t *)calloc(1, sizeof(Node_t));
    if (!node) return NULL;

    node->name_len = strlen(name);
    node->name = (char *)calloc(node->name_len, sizeof(char));
    strncpy(node->name, name, node->name_len);

    // Create a sub-directory for that specific node.
    mkdir(join_paths(2, NODE_DIR, name), 0777);

    node->pid = 0;

    return node;
}

void kill_node(Node_t* node)
{
    // Kill node (stop it from running)
    kill(node->pid, SIGINT);
    usleep(PROC_CLEANUP_DURATION_US); // Wait 1 ms for process cleanup
    if (kill(node->pid, 0) == 0) kill(node->pid, SIGKILL); // Process took to long to clean up. Kill it.
}

void run_node(Node_t* node)
{
    int pid = fork();
    if (pid == 0)
    {
        const char* path = join_paths(4, NODE_DIR, node->name, "build", "main");
        execlp(path, path);
    }
}

void free_node(Node_t* node)
{
    kill_node(node);

    rmdir(join_paths(2, NODE_DIR, node->name));
    free(node->name);
    free(node);
}

void restart_node(Node_t* node)
{
    kill_node(node);
    run_node(node);
}

#ifdef __cplusplus
}
#endif