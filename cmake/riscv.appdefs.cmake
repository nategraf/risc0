# This file sets a default compiler flags for the cross builds 

add_compile_options(
  -DRISCV=1
  -mabi=ilp32
  -march=rv32im
  -ffreestanding
  -fno-strict-aliasing
  -fno-exceptions
  -fno-non-call-exceptions
  -Wall
  -Wunused-but-set-parameter
  -Wno-error=pragmas
  -Wno-unknown-pragmas
  -Wno-strict-aliasing
  -fdata-sections
  -ffunction-sections
  -findirect-inlining
  -finline-small-functions
  -g0
  -O2
  -fno-rtti
  -fno-threadsafe-statics
  -fno-use-cxa-atexit
  # -MD -MF
)

add_link_options(
  -DRISCV=1
  -mabi=ilp32
  -march=rv32im
  -ffreestanding
  -fno-strict-aliasing
  -nostdlib
  -Wl,--gc-sections
  -T
  ${CMAKE_SOURCE_DIR}/risc0/zkvm/platform/risc0.ld
  -lc
  -lgcc
  -lsemihost)
