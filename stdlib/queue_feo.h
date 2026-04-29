#ifndef __QUEUE_H__
#define __QUEUE_H__

#include <stddef.h>
#include <stdlib.h>

#ifdef __cplusplus
extern "C"
{
#endif

typedef struct __QueueNode
{
    struct __QueueNode* next;
    void* value;
} __QueueNode;

typedef struct queue_t
{
    __QueueNode* front;
    __QueueNode* back;
    size_t length;
} queue_t;

/* Create a new queue. */
queue_t* new_queue()
{
    queue_t* q_ = (queue_t *)calloc(1, sizeof(queue_t));
    if (!q_) return NULL;

    q_->front = NULL;
    q_->back = NULL;
    q_->length = 0;

    return q_;
}

/* Free a queue */
void free_queue(queue_t* q_)
{
    __QueueNode* curr = q_->front;
    __QueueNode* next = q_->front->next;
    free(curr);
    while (next)
    {
        curr = next;
        next = next->next;
        free(curr);
    }

    free(q_);
}

/* Pop a value from the front of a queue and return it. */
void* pop_queue(queue_t* q_)
{
    __QueueNode* next = q_->front->next;
    void* curr_value = q_->front->value;
    free(q_->front);
    q_->front = next;
    q_->length --;
    return curr_value;
}

/* Push a value to the back of a queue. */
size_t push_queue(queue_t* q_, void* data)
{
    __QueueNode* node = (__QueueNode *)calloc(1, sizeof(__QueueNode));
    node->value = data;
    node->next = NULL; // Very important for deletion
    q_->back->next = node;
    q_->back = q_->back->next;
    return ++ q_->length;
}

/* Append a queue to the end  of the current one. */
void add_queue(queue_t* q1, queue_t* q2)
{
    q1->back->next = q2->front;
    q1->back = q2->back;
}

/* Return value at front of queue. */
inline void* front_queue(queue_t* q_)
{
    return q_->front->value;
}

/* Return value at back of queue. */
inline void* back_queue(queue_t* q_)
{
    return q_->back->value;
}

#ifdef __cplusplus
}
#endif

#endif