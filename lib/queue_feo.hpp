#pragma once

template <typename T>
class queue
{
    struct Node
    {
        Node* next;
        T value;
    };

    Node* front;
    Node* back;
    size_t length;

public:
    queue()
    {
        front = nullptr;
        back = nullptr;
        length = 0;
    }

    ~queue()
    {
        Node* curr = front;
        Node* next = front->next;
        delete curr;
        while (next)
        {
            curr = next;
            next = next->next;
            delete curr;
        }

        length = 0;
    }

    size_t push(const T& value)
    {
        Node* node = new Node;
        node->value = value;
        node->next = nullptr; // Very important for deletion
        back->next = node;
        back = back->next;
        return ++ length;
    }

    const T pop()
    {
        Node* next = front->next;
        T value = front->value;
        delete front;
        return value;
    }

    inline const T front() { return front->value; }
    inline const T back() { return back->value; }

    queue operator + (const T& other)
    {
        Node* node = new Node;
        node->value = value;
        node->next = nullptr; // Very important for deletion
        back->next = node;
        back = back->next;
        return *this;
    }

    queue operator + (const queue& other)
    {
        back->next = other->front;
        length += other.length;
        other.length = 0;
        return *this;
    }

    void operator += (const T& other)
    {
        Node* node = new Node;
        node->value = value;
        node->next = nullptr; // Very important for deletion
        back->next = node;
        back = back->next;
    }

    void operator += (const queue& other)
    {
        back->next = other->front;
        length += other.length;
        other.length = 0;
    }
};
