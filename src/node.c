#include <node.h>
#include <fcntl.h>

Node_t procs[MAX_NUM_PROCS];

void init_nodes()
{
    mkdir(NODE_DIR, 0777);
}

Node_t* create_node(const char* name)
{
    Node_t* node = (Node_t *)calloc(1, sizeof(Node_t));
    if (!node)
    {
        __global_err = ERR_FAILED_MALLOC;
        return NULL;
    }

    node->name_len = strlen(name);
    node->name = (char *)calloc(node->name_len + 1, sizeof(char));
    if (!node->name)
    {
        __global_err = ERR_FAILED_MALLOC;
        return NULL;
    }

    strncpy(node->name, name, node->name_len);
    node->name[node->name_len] = '\0';

    // Create a sub-directory for that specific node.
    char* path = join_paths(2, NODE_DIR, name);
    mkdir(path, 0777);
    free(path);

    node->pid = 0;

    return node;
}

void compile_node(Node_t* node)
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

    __global_err = ERR_OK;
}

void run_node(Node_t* node)
{
    int pid = fork();
    if (pid == 0)
    {
        // Executable produced by `cargo build --release --target-dir <target_dir>`
        // will be at: <target_dir>/release/<crate_name>
        char* exe_path = join_paths(5, NODE_DIR, node->name, "build", "release", node->name);

        // Prepare logs directory and files: <target_dir>/logs/stdout.log, stderr.log
        char* logs_dir = join_paths(4, NODE_DIR, node->name, "build", "logs");
        mkdir(logs_dir, 0777);

        char* stdout_path = join_paths(5, NODE_DIR, node->name, "build", "logs", "stdout.log");
        char* stderr_path = join_paths(5, NODE_DIR, node->name, "build", "logs", "stderr.log");

        int out_fd = open(stdout_path, O_WRONLY | O_CREAT | O_APPEND, 0644);
        if (out_fd >= 0) {
            dup2(out_fd, STDOUT_FILENO);
            close(out_fd);
        }

        int err_fd = open(stderr_path, O_WRONLY | O_CREAT | O_APPEND, 0644);
        if (err_fd >= 0) {
            dup2(err_fd, STDERR_FILENO);
            close(err_fd);
        }

        free(logs_dir);
        free(stdout_path);
        free(stderr_path);

        execlp(exe_path, exe_path, NULL);
        // If execlp returns, log errno to stderr (redirected to stderr.log) then exit
        perror("execlp failed");
        free(exe_path);
        _exit(ERR_EXECLP);
    }

    // Clear `__global_err` to check fro erros in `register_process`
    __global_err = ERR_OK;

    // Register the node
    node->pid = pid;
    register_process(node);

    // Too many nodes. Free it as the system can't track more
    if (__global_err == ERR_MAX_PROC_REACHED)
        free_node(node);
}

void kill_node(Node_t* node)
{
    // Kill node (stop it from running)
    kill(node->pid, SIGINT);
    usleep(PROC_CLEANUP_DURATION_US); // Wait 1 ms for process cleanup
    if (kill(node->pid, 0) == 0)
    {
        // Process took to long to clean up. Kill it.
        kill(node->pid, SIGKILL);
    }
}

void free_node(Node_t* node)
{
    kill_node(node);

    char* path = join_paths(2, NODE_DIR, node->name);
    rmdir(path);
    free(path);
    free(node->name);
    free(node);
}

void restart_node(Node_t* node)
{
    kill_node(node);
    run_node(node);
}

int* reap_processes()
{
    int status;
    pid_t pid;

    int* ret = calloc(MAX_NUM_PROCS, sizeof(int));
    if (! ret)
    {
        __global_err = ERR_FAILED_MALLOC;
        return NULL;
    }

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

void register_process(Node_t* node)
{
    for (int i = 0; i < MAX_NUM_PROCS; i++)
    {
        if (!procs[i].active)
        {
            procs[i].pid = node->pid;
            procs[i].active = 1;
            return;
        }
    }

    __global_err = ERR_MAX_PROC_REACHED;
}
