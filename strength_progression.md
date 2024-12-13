# Initial Implementation

The first implementation of the minimax algorithm. Not done very efficiently.
There is also lots of `clone` calls that are likely slowing the algorithm down.
It also only looks two moves ahead right now, due to performance constraints.

Can I beat this version? Yes. Early game the bot just moves pieces back and forth,
allowing me plenty of time to set up a devestating attack. That said, during the
attack the bot is pretty hard to checkmate. It finds defensive moves that I did not
consider. But overall, not that strong. But likely already strong enough to beat beginners.

# Happy squares

After adding happy squares, the bot outplayed me in the midgame! With only a depth of two, I'm quite impressed.
I did not set up the happy squares for pawns correctly and it stil did well overall.
