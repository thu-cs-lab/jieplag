#include <stdio.h>

int aplusb(int a, int b) {
  // implement aplusb
  return a+b;
}

int main() {
  // implement input/output
  int a, b;
  scanf("%d%d", &a, &b);
  int c = aplusb(a, b);
  printf("%d\n", c);
  return 0;
}