#ifndef SORT_KEY_HPP
#define SORT_KEY_HPP

#include "list.hpp"

class SortKey {
public:
    SortKey& operator= (const SortKey& other);
    SortKey(const SortKey &other);
    ~SortKey() { }

    static SortKey single(const SortKey *low, const SortKey *high);
    static void multi(List<SortKey> &out_sort_key_list, const SortKey *low, const SortKey *high, int count);
    static int compare(const SortKey &a, const SortKey &b);

private:
    SortKey();
    SortKey(int value);

    int magnitude;
    List<uint8_t> digits;

    static SortKey average(const SortKey &low, const SortKey &high);
    static SortKey increment(const SortKey &value);
    static SortKey add(const SortKey &a, const SortKey &b);

    static void pad_to_equal_magnitude(SortKey &a, SortKey &b);
    static void pad_in_place(SortKey &sort_key, int magnitude);
    static void truncate_fraction(SortKey &value);
};

#endif

