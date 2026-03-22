#include <stdio.h>
#include <stdbool.h>
#include "src/editor_abi.h"

int main(void) {
    printf("🎯 Features: 0x%x\n", editor_get_features());
    return 0;
}
