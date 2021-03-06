#include "id_map.hpp"

// this can't be inline static, or else any global variables that use it will
// secretly become static, even if you declare them with extern.
uint32_t hash_uint256(const uint256 & a) {
    return UIntOversized<4>::hash(a);
}
