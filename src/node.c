#include <node.h>

Node_t procs[MAX_NUM_PROCS];

int init_nodes()
{
    mkdir(NODE_DIR, 0777);
    return ERR_OK;
}

Node_t* create_node(const char* name)
{
    Node_t* node = (Node_t *)calloc(1, sizeof(Node_t));
    if (!node) return NULL;

    node->name_len = strlen(name);
    node->name = (char *)calloc(node->name_len + 1, sizeof(char));
    strncpy(node->name, name, node->name_len);
    node->name[node->name_len] = '\0';

    // Create a sub-directory for that specific node.
    char* path = join_paths(2, NODE_DIR, name);
    mkdir(path, 0777);
    free(path);

    node->pid = 0;

    return node;
}

int compile_node(Node_t* node)
{
    char* path = join_paths(2, "/home/feo/nodes", node->name);
    char* target_dir = join_paths(3, NODE_DIR, node->name, "build");

    pid_t pid = fork();
    if (pid == 0)
    {
        chdir(path);

        execlp(
            "cargo",
            "cargo",
            "build",
            "--release",
            "--target-dir",
            target_dir,
            NULL
        );

        _exit(1);
    }

    free(path);
    free(target_dir);

    return ERR_OK;
}

int run_node(Node_t* node)
{
    int ret = ERR_OK;
    int pid = fork();
    if (pid == 0)
    {
        const char* path = join_paths(4, NODE_DIR, node->name, "build", "target", "main");
        execlp(path, path, NULL);
        _exit(ERR_EXECLP);
    }

    // Register the node
    node->pid = pid;
    ret = register_process(node);

    // Too many nodes. Free it as the system can't track more
    if (ret == ERR_MAX_PROC_REACHED)
        free_node(node);

    return ret;
}

int kill_node(Node_t* node)
{
    // Kill node (stop it from running)
    kill(node->pid, SIGINT);
    usleep(PROC_CLEANUP_DURATION_US); // Wait 1 ms for process cleanup
    if (kill(node->pid, 0) == 0)
    {
        // Process took to long to clean up. Kill it.
        kill(node->pid, SIGKILL);
        return 1;
    }

    return ERR_OK;
}

int free_node(Node_t* node)
{
    int ret = ERR_OK;
    ret = kill_node(node);

    char* path = join_paths(2, NODE_DIR, node->name);
    rmdir(path);
    free(path);
    free(node->name);
    free(node);

    return ret;
}

int restart_node(Node_t* node)
{
    kill_node(node);
    run_node(node);

    return ERR_OK;
}

int* reap_processes()
{
    int status;
    pid_t pid;

    int* ret = calloc(MAX_NUM_PROCS, sizeof(int));

    while ((pid = waitpid(-1, &status, WNOHANG)) > 0)
    {
        for (int i = 0; i < MAX_NUM_PROCS; i++)
        {
            if (procs[i].active && procs[i].pid == pid)
            {
                procs[i].active = 0;
                ret[i] = 1; // Mark as dead
                break;
            }
        }
    }

    return ret;
}

int register_process(Node_t* node)
{
    for (int i = 0; i < MAX_NUM_PROCS; i++)
    {
        if (!procs[i].active)
        {
            procs[i].pid = node->pid;
            procs[i].active = 1;
            return ERR_OK;
        }
    }

    return ERR_MAX_PROC_REACHED;
}
