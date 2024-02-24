## cr8: a chip8 assembler
written in rust  
lexing and parsing using [plex](https://github.com/goffrie/plex/)

## usage
```console
$ cargo run -- input.cr8 output.ch8
```

## language manual
still in progress!
```coffeescript
# this is a comment
clear           # clear the screen
vx = xy         # copy register value
vx = n          # set register value
vx += n         # increment register value
i = n           # set i value
i = *vx         # set i to sprite position of register
draw vx vy n    # draw at position vx vy and height n from sprite position i
main:           # declare label
goto main       # go to label
```

## also see
- [cheap: a chip8 emulator](https://github.com/lumixing/cheap/)
- [chip8 technical reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM/)
- [chip8 book](https://github.com/aquova/chip8-book/)
- [octo](https://github.com/JohnEarnest/Octo/)
