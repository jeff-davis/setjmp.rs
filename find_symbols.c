#include <setjmp.h>

int main()
{
  jmp_buf jmpbuf;
  sigjmp_buf sigjmpbuf;
  setjmp(jmpbuf);
  sigsetjmp(sigjmpbuf, 0);
  longjmp(jmpbuf, 1);
  siglongjmp(sigjmpbuf, 1);
}
