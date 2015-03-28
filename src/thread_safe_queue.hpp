#ifndef THREAD_SAFE_QUEUE
#define THREAD_SAFE_QUEUE

#include "error.h"

#include <linux/futex.h>
#include <sys/time.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <sys/errno.h>

#include <atomic>
using std::atomic_int;
using std::atomic_flag;
#ifndef ATOMIC_INT_LOCK_FREE
#error "require atomic int to be lock free"
#endif

// hopefully if this is true, we can send the address of an atomic int to the futex
// syscall and have it work correctly
static_assert(sizeof(int) == sizeof(atomic_int), "require atomic_int to be same size as int");

static int futex(int *uaddr, int op, int val, const struct timespec *timeout, int *uaddr2, int val3) {
    return syscall(SYS_futex, uaddr, op, val, timeout, uaddr2, val3);
}

static int futex_wait(int *address, int val) {
    return futex(address, FUTEX_WAIT, val, nullptr, nullptr, 0) ? errno : 0;
}

static int futex_wake(int *address, int count) {
    return futex(address, FUTEX_WAKE, count, nullptr, nullptr, 0) ? errno : 0;
}

// many writer, many reader, fixed size, thread-safe, first-in-first-out queue
// lock-free except when a reader calls dequeue() and queue is empty, then it blocks
// must call resize before you can start using it
template<typename T>
class ThreadSafeQueue {
public:
    ThreadSafeQueue() {
        _slots = nullptr;
        _size = 0;
    }
    ~ThreadSafeQueue() {
        destroy(_slots, size);
    }

    // this method not thread safe
    int __attribute__((warn_unused_result)) resize(int size) {
        if (length < 0)
            return GenesisErrorInvalidParam;

        T *new_items = reallocate_safe(_slots, _size, size);
        if (!new_items)
            return GenesisErrorNoMem;

        _size = size;
        _queue_count = 0;
        _read_index = 0;
        _write_index = 0;
        _modulus_flag = ATOMIC_FLAG_INIT;

        return 0;
    }

    void enqueue(T item) {
        int my_write_index = _write_index.fetch_add(1);
        int in_bounds_index = my_write_index % _size;
        Slot *slot = &_slots[in_bounds_index];
        slot->item = item;
        int my_queue_count = _queue_count.fetch_add(1);
        if (my_queue_count >= _size)
            panic("queue is full");
        if (my_queue_count <= 0)
            futex_wake(&_queue_count, 1);
    }

    T dequeue() {
outer:
        int my_queue_count = _queue_count.fetch_and_sub(1);
        if (my_queue_count <= 0) {
            // need to block because there are no items in the queue
            for (;;) {
                int err = futex_wait(&_queue_count, my_queue_count - 1);
                if (err == EACCES || err == EINVAL || err == ENOSYS) {
                    panic("futex wait error");
                } else if (err == EWOULDBLOCK) {
                    // waiting failed because _queue_count changed.
                    // release the changed state and then try again
                    _queue_count += 1;
                    goto outer;
                } else if (err == EINTR) {
                    // spurious wakeup
                    continue;
                } else {
                    // normal wakeup. continue the dequeue
                    break;
                }
            }
        }

        int my_read_index = _read_index.fetch_add(1);
        int in_bounds_index = my_read_index % _size;
        // keep the index values in check
        if (my_read_index >= _size && !_modulus_flag.test_and_set()) {
            _read_index -= _size;
            _write_index -= _size;
            _modulus_flag.clear();
        }
        Slot *slot = _slots[in_bounds_index];
        return slot->item;
    }

    void wakeup_all() {
        futex_wake(&_queue_count, -_queue_count);
    }

private:
    struct Slot {
        T item;
    };

    Slot *_slots;
    int _size;
    atomic_int _queue_count;
    atomic_int _read_index;
    atomic_int _write_index;
    atomic_flag _modulus_flag;
};

#endif