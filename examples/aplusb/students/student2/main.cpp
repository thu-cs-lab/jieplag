#include <iostream>

int aplusb(int a, int b) {
  // implement aplusb
  int c = a + b;
  return c;
}

int main() {
  // implement input/output
  int a, b;
  std::cin >> a >> b;
  std::cout << aplusb(a, b);
  return 0;
}