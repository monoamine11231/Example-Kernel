int _start() {

    char string[] = "Hello from kernel C";
    char* vid_mem = (char*)0xb8000;

    char *c = &string[0];
    while (*c) {
        *(vid_mem+0) = *c;
        *(vid_mem+1) = ' ';

        vid_mem += 2; 
        c++;
    }

    for(;;) {

    }
}