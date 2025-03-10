# zeus

zeus is an operating system for riscv64.

## Features
 - Single Kernel Stack
 - User processes run in supervisor mode (yes, that is a feature)
 - No paging, because its cool when userspace programs can crash the kernel
 - Syscalls through calling memory address (0x80200010), because who needs ecall anyways?


## License

zeus is licensed under the MIT license.


