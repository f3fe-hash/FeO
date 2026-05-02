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
#include <sys/wait.h>

// Custom Utilities
#include <utils.h>

#define PROC_CLEANUP_DURATION_US 1000 // Wait 1 ms for process cleanup before forcefully killing it.

#define NODE_DIR "/etc/nodes"

#define MAX_NUM_PROCS 128

typedef struct feo_node_t
{
    pid_t pid;
    int active;

    char* name;
    int name_len;
} Node_t;

extern Node_t procs[MAX_NUM_PROCS];

int init_nodes();

Node_t* create_node(const char* name);
int compile_node(Node_t* node);
int run_node(Node_t* node);
int kill_node(Node_t* node);
int free_node(Node_t* node);
int restart_node(Node_t* node);

int* reap_processes();
int register_process(Node_t* node);

#ifdef __cplusplus
}
#endif