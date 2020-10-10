_Note: prototype work in progress, not intended for public use_

# rp2g

> Remote Play Together any Steam game.

Many Steam games have [_Remote Play Together_][steam-page] disabled.
That's stupid.
This tool enables you to use Remote Play Together with _any_ Steam game (or
other program).

This can be achieved manually by moving game files around. That's very
cumbersome.
This tool automates the whole process in a smart way and makes this a breeze.

##### Side effects
- Steam will think you're playing a different game
- Online play and other Steam features might not work

##### Platform support
- Linux x86_64

##### How does it work
- Tool picks a placeholder game that supports Remote Play Together
- Tool prepares your game or program to start, faking it is the placeholder game
- Tool starts game and you can Remote Play Together

## Usage
```bash
rp2g [GAME]
```

## License
This project is released under the GNU GPL-3.0 license.
Check out the [LICENSE](LICENSE) file for more information.

[steam-page]: https://store.steampowered.com/remoteplay/#together
