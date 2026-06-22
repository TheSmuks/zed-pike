#!/usr/bin/env pike

// Basic Pike fixture for the Zed extension.
#define ANSWER 42

class Demo {
  public int add(int a, int b) {
    return a + b;
  }

  string greet(string name) {
    return "Hello, " + name + "\n";
  }
}

int main(int argc, array(string) argv) {
  Demo demo = Demo();
  write("%d\n", demo->add(ANSWER, 1));
  return 0;
}
