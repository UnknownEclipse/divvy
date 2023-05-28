#include <new>

extern "C" {
void *divvy_cpp_alloc(size_t size, size_t align) {
  return ::operator new(size, static_cast<std::align_val_t>(align),
                        std::nothrow);
}

void divvy_cpp_dealloc(void *ptr, size_t size, size_t align) {
  ::operator delete(ptr, size, static_cast<std::align_val_t>(align));
}
}
